//! GPS route simplification using the Ramer-Douglas-Peucker algorithm
//!
//! Reduces the number of points in a GPS track while preserving the overall
//! route shape. Returns indices of points to keep rather than copying data,
//! which preserves all metadata from the original activity stream.

use crate::database::entities::activity_stream;
use std::f64::consts::PI;

/// Earth's mean radius approximation: meters per degree of latitude
/// This value is constant globally (~111.32 km per degree)
const METERS_PER_DEGREE_LAT: f64 = 111_320.0;

/// Errors that can occur during route simplification
#[derive(Debug, thiserror::Error)]
pub enum SimplificationError {
    #[error("Epsilon must be positive, got {0}")]
    InvalidEpsilon(f64),

    #[error("No valid GPS coordinates found in activity stream")]
    NoGpsCoordinates,
}

/// Internal representation of a GPS coordinate for calculations
#[derive(Debug, Clone, Copy)]
struct GpsPoint {
    lat: f64,
    lng: f64,
}

impl GpsPoint {
    fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }
}

/// Simplifies a GPS route using the Ramer-Douglas-Peucker algorithm
///
/// Returns a vector of indices into the original points slice that should be kept.
/// First and last indices are always included. Uses an iterative approach to avoid
/// stack overflow on large routes.
///
/// # Arguments
///
/// * `points` - Slice of activity stream models with GPS coordinates
/// * `epsilon` - Maximum perpendicular distance threshold in meters
///
/// # Returns
///
/// Vector of indices to keep from the original points slice, sorted in ascending order.
/// These indices can be used to filter the original slice while preserving all metadata
/// (time, `heart_rate`, cadence, etc.).
///
/// # Errors
///
/// Returns error if:
/// - `epsilon` is negative, zero, or NaN
/// - No valid GPS coordinates found in input (all lat/lng are None)
///
/// # Example
///
/// ```rust,ignore
/// use run_sous_bpm_core::geo::simplify_gps_route;
///
/// let streams = get_activity_streams(db, activity_id).await?;
/// let indices = simplify_gps_route(&streams, 10.0)?; // 10 meter tolerance
/// let simplified: Vec<_> = indices.iter().map(|&i| &streams[i]).collect();
/// ```
pub fn simplify_gps_route(
    points: &[activity_stream::Model],
    epsilon: f64,
) -> Result<Vec<usize>, SimplificationError> {
    // Validate epsilon
    if epsilon <= 0.0 || epsilon.is_nan() {
        return Err(SimplificationError::InvalidEpsilon(epsilon));
    }

    // Extract GPS points and build index mapping
    let (gps_points, index_map) = extract_gps_points(points);

    // Handle edge cases
    if gps_points.len() < 2 {
        return Err(SimplificationError::NoGpsCoordinates);
    }

    if gps_points.len() == 2 {
        // Only two points - keep both
        return Ok(vec![index_map[0], index_map[1]]);
    }

    // Run RDP algorithm
    let keep_flags = rdp_iterative(&gps_points, epsilon);

    // Convert keep flags to original indices
    let result: Vec<usize> = keep_flags
        .iter()
        .enumerate()
        .filter_map(|(i, &keep)| if keep { Some(index_map[i]) } else { None })
        .collect();

    Ok(result)
}

/// Extracts valid GPS points from activity stream models
///
/// Returns a tuple of (GPS points, index mapping). The index mapping
/// allows us to convert from filtered indices back to original indices.
///
/// # Arguments
///
/// * `points` - Slice of activity stream models
///
/// # Returns
///
/// Tuple of (Vec<GpsPoint>, Vec<usize>) where the second element maps
/// filtered index -> original index
fn extract_gps_points(points: &[activity_stream::Model]) -> (Vec<GpsPoint>, Vec<usize>) {
    points
        .iter()
        .enumerate()
        .filter_map(|(i, model)| match (model.latitude, model.longitude) {
            (Some(lat), Some(lng)) => Some((GpsPoint::new(lat, lng), i)),
            _ => None,
        })
        .unzip()
}

/// Iterative implementation of the Ramer-Douglas-Peucker algorithm
///
/// Uses an explicit stack to avoid stack overflow on large routes.
///
/// # Arguments
///
/// * `points` - Slice of GPS points to simplify
/// * `epsilon` - Maximum perpendicular distance threshold in meters
///
/// # Returns
///
/// Vector of boolean flags indicating which points to keep
fn rdp_iterative(points: &[GpsPoint], epsilon: f64) -> Vec<bool> {
    let n = points.len();
    let mut keep = vec![false; n];

    // Always keep first and last points
    keep[0] = true;
    keep[n - 1] = true;

    // Stack of (start_index, end_index) segments to process
    let mut stack = vec![(0, n - 1)];

    while let Some((start, end)) = stack.pop() {
        if end - start <= 1 {
            // No points between start and end
            continue;
        }

        // Find point with maximum perpendicular distance
        let (max_idx, max_dist) = find_farthest_point(points, start, end);

        if max_dist > epsilon {
            // Keep this point and recursively process both segments
            keep[max_idx] = true;
            stack.push((start, max_idx));
            stack.push((max_idx, end));
        }
    }

    keep
}

/// Finds the point with maximum perpendicular distance to a line segment
///
/// # Arguments
///
/// * `points` - Slice of GPS points
/// * `start` - Start index of line segment
/// * `end` - End index of line segment
///
/// # Returns
///
/// Tuple of (index of farthest point, distance in meters)
fn find_farthest_point(points: &[GpsPoint], start: usize, end: usize) -> (usize, f64) {
    let mut max_dist = 0.0;
    let mut max_idx = start;

    let line_start = points[start];
    let line_end = points[end];

    for (i, &point) in points.iter().enumerate().take(end).skip(start + 1) {
        let dist = perpendicular_distance(point, line_start, line_end);
        if dist > max_dist {
            max_dist = dist;
            max_idx = i;
        }
    }

    (max_idx, max_dist)
}

/// Calculates perpendicular distance from a point to a line segment
///
/// Uses the cross product formula to compute the perpendicular distance,
/// then converts from degrees to meters using equirectangular projection.
///
/// # Arguments
///
/// * `point` - The point to measure distance from
/// * `line_start` - Start of the line segment
/// * `line_end` - End of the line segment
///
/// # Returns
///
/// Perpendicular distance in meters
fn perpendicular_distance(point: GpsPoint, line_start: GpsPoint, line_end: GpsPoint) -> f64 {
    // Vector from line_start to line_end
    let line_vec = (line_end.lng - line_start.lng, line_end.lat - line_start.lat);

    // Vector from line_start to point
    let point_vec = (point.lng - line_start.lng, point.lat - line_start.lat);

    // Cross product gives signed area of parallelogram
    let cross = point_vec.0 * line_vec.1 - point_vec.1 * line_vec.0;

    // Length of line segment in degrees
    let line_len_deg = (line_vec.0 * line_vec.0 + line_vec.1 * line_vec.1).sqrt();

    if line_len_deg < 1e-10 {
        // Line segment is essentially a point, return distance to that point
        return equirectangular_distance(point, line_start);
    }

    // Perpendicular distance in degrees
    let dist_deg = cross.abs() / line_len_deg;

    // Convert to meters using average latitude
    let avg_lat = f64::midpoint(line_start.lat, line_end.lat);
    degrees_to_meters(dist_deg, avg_lat)
}

/// Calculates distance between two GPS points using equirectangular projection
///
/// This is 30x faster than Haversine with <0.5% error for typical activity
/// distances (<100km). Perfect for route simplification where we only need
/// relative distance comparisons.
///
/// # Arguments
///
/// * `p1` - First GPS point
/// * `p2` - Second GPS point
///
/// # Returns
///
/// Distance in meters
fn equirectangular_distance(p1: GpsPoint, p2: GpsPoint) -> f64 {
    let avg_lat_rad = f64::midpoint(p1.lat, p2.lat) * PI / 180.0;
    let meters_per_degree_lng = METERS_PER_DEGREE_LAT * avg_lat_rad.cos();

    let dx = (p2.lng - p1.lng) * meters_per_degree_lng;
    let dy = (p2.lat - p1.lat) * METERS_PER_DEGREE_LAT;

    (dx * dx + dy * dy).sqrt()
}

/// Converts a distance in degrees to meters
///
/// # Arguments
///
/// * `degrees` - Distance in degrees
/// * `latitude` - Latitude at which to calculate (affects longitude scaling)
///
/// # Returns
///
/// Distance in meters
fn degrees_to_meters(degrees: f64, latitude: f64) -> f64 {
    let lat_rad = latitude * PI / 180.0;
    let meters_per_degree_avg = ((METERS_PER_DEGREE_LAT * METERS_PER_DEGREE_LAT)
        + (METERS_PER_DEGREE_LAT * lat_rad.cos()) * (METERS_PER_DEGREE_LAT * lat_rad.cos()))
    .sqrt();
    degrees * meters_per_degree_avg
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    /// Helper to create a test activity stream model with GPS coordinates
    fn make_point(lat: f64, lng: f64) -> activity_stream::Model {
        activity_stream::Model {
            activity_id: uuid::Uuid::new_v4(),
            time: DateTime::from_timestamp(0, 0).unwrap().into(),
            latitude: Some(lat),
            longitude: Some(lng),
            altitude: None,
            heart_rate: None,
            cadence: None,
            watts: None,
            velocity: None,
            distance: None,
            temperature: None,
        }
    }

    #[test]
    fn test_empty_slice() {
        let points: Vec<activity_stream::Model> = vec![];
        let result = simplify_gps_route(&points, 10.0);
        assert!(matches!(result, Err(SimplificationError::NoGpsCoordinates)));
    }

    #[test]
    fn test_single_point() {
        let points = vec![make_point(48.8566, 2.3522)];
        let result = simplify_gps_route(&points, 10.0);
        assert!(matches!(result, Err(SimplificationError::NoGpsCoordinates)));
    }

    #[test]
    fn test_two_points() {
        let points = vec![make_point(48.8566, 2.3522), make_point(48.8567, 2.3523)];
        let result = simplify_gps_route(&points, 10.0).unwrap();
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn test_invalid_epsilon_negative() {
        let points = vec![make_point(48.8566, 2.3522), make_point(48.8567, 2.3523)];
        let result = simplify_gps_route(&points, -10.0);
        assert!(matches!(
            result,
            Err(SimplificationError::InvalidEpsilon(_))
        ));
    }

    #[test]
    fn test_invalid_epsilon_zero() {
        let points = vec![make_point(48.8566, 2.3522), make_point(48.8567, 2.3523)];
        let result = simplify_gps_route(&points, 0.0);
        assert!(matches!(
            result,
            Err(SimplificationError::InvalidEpsilon(_))
        ));
    }

    #[test]
    fn test_invalid_epsilon_nan() {
        let points = vec![make_point(48.8566, 2.3522), make_point(48.8567, 2.3523)];
        let result = simplify_gps_route(&points, f64::NAN);
        assert!(matches!(
            result,
            Err(SimplificationError::InvalidEpsilon(_))
        ));
    }

    #[test]
    fn test_no_gps_coordinates() {
        let mut points = vec![make_point(48.8566, 2.3522)];
        points[0].latitude = None;
        points[0].longitude = None;
        let result = simplify_gps_route(&points, 10.0);
        assert!(matches!(result, Err(SimplificationError::NoGpsCoordinates)));
    }

    #[test]
    fn test_straight_line_collinear() {
        // Three points in a straight line
        let points = vec![
            make_point(48.0, 2.0),
            make_point(48.1, 2.1),
            make_point(48.2, 2.2),
        ];

        // With high tolerance, should keep only endpoints
        let result = simplify_gps_route(&points, 100.0).unwrap();
        assert_eq!(result, vec![0, 2]);
    }

    #[test]
    fn test_triangle_all_kept() {
        // Three points forming a triangle
        let points = vec![
            make_point(48.0, 2.0),
            make_point(48.1, 2.0),
            make_point(48.0, 2.1),
        ];

        // With low tolerance, should keep all points
        let result = simplify_gps_route(&points, 1.0).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&0));
        assert!(result.contains(&1));
        assert!(result.contains(&2));
    }

    #[test]
    fn test_zigzag_reduction() {
        // Zig-zag pattern
        let points = vec![
            make_point(48.0, 2.0),
            make_point(48.01, 2.01),
            make_point(48.02, 2.0),
            make_point(48.03, 2.01),
            make_point(48.04, 2.0),
        ];

        // High tolerance should reduce to endpoints
        let result = simplify_gps_route(&points, 2000.0).unwrap();
        assert_eq!(result, vec![0, 4]);

        // Low tolerance should keep more points
        let result_low = simplify_gps_route(&points, 10.0).unwrap();
        assert!(result_low.len() > 2);
    }

    #[test]
    fn test_sparse_gps_data() {
        // Some points without GPS coordinates
        let mut points = vec![
            make_point(48.0, 2.0),
            make_point(48.1, 2.1),
            make_point(48.2, 2.2),
            make_point(48.3, 2.3),
        ];

        // Remove GPS from middle point
        points[1].latitude = None;
        points[1].longitude = None;

        let result = simplify_gps_route(&points, 10.0).unwrap();

        // Should work with remaining valid points
        // Indices should map to original positions (0, 2, 3)
        assert!(result.contains(&0));
        assert!(result.contains(&3));
    }

    #[test]
    fn test_equirectangular_distance() {
        // Paris coordinates
        let p1 = GpsPoint::new(48.8566, 2.3522);
        // Approximately 1km east
        let p2 = GpsPoint::new(48.8566, 2.3662);

        let dist = equirectangular_distance(p1, p2);

        // Should be approximately 1000 meters (within 5% error)
        assert!((dist - 1000.0).abs() < 50.0, "Distance: {dist}");
    }

    #[test]
    fn test_always_keeps_first_and_last() {
        let points = vec![
            make_point(48.0, 2.0),
            make_point(48.1, 2.1),
            make_point(48.2, 2.2),
            make_point(48.3, 2.3),
            make_point(48.4, 2.4),
        ];

        let result = simplify_gps_route(&points, 1000.0).unwrap();

        // Even with very high tolerance, first and last are always kept
        assert_eq!(result[0], 0);
        assert_eq!(result[result.len() - 1], 4);
    }
}
