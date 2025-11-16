/**
 * Douglas-Peucker algorithm for simplifying GPS routes
 * Reduces the number of points while preserving the overall shape
 */

// TODO: Might be moved to backend

interface Point {
  latitude: number;
  longitude: number;
}

/**
 * Calculate perpendicular distance from a point to a line segment
 */
function perpendicularDistance(
  point: Point,
  lineStart: Point,
  lineEnd: Point
): number {
  const dx = lineEnd.longitude - lineStart.longitude;
  const dy = lineEnd.latitude - lineStart.latitude;

  // Normalize
  const mag = Math.sqrt(dx * dx + dy * dy);
  if (mag > 0) {
    const u =
      ((point.longitude - lineStart.longitude) * dx +
        (point.latitude - lineStart.latitude) * dy) /
      (mag * mag);

    const intersectionX = lineStart.longitude + u * dx;
    const intersectionY = lineStart.latitude + u * dy;

    const distX = point.longitude - intersectionX;
    const distY = point.latitude - intersectionY;

    return Math.sqrt(distX * distX + distY * distY);
  }

  // Fallback to distance to start point
  const distX = point.longitude - lineStart.longitude;
  const distY = point.latitude - lineStart.latitude;
  return Math.sqrt(distX * distX + distY * distY);
}

/**
 * Recursive Douglas-Peucker implementation
 */
function douglasPeuckerRecursive(
  points: Point[],
  epsilon: number,
  start: number,
  end: number,
  keep: boolean[]
): void {
  let maxDistance = 0;
  let maxIndex = 0;

  // Find the point with maximum distance from line segment
  for (let i = start + 1; i < end; i++) {
    const distance = perpendicularDistance(
      points[i],
      points[start],
      points[end]
    );

    if (distance > maxDistance) {
      maxDistance = distance;
      maxIndex = i;
    }
  }

  // If max distance is greater than epsilon, recursively simplify
  if (maxDistance > epsilon) {
    keep[maxIndex] = true;

    douglasPeuckerRecursive(points, epsilon, start, maxIndex, keep);
    douglasPeuckerRecursive(points, epsilon, maxIndex, end, keep);
  }
}

/**
 * Simplify a route using Douglas-Peucker algorithm
 *
 * @param points Array of GPS coordinates
 * @param tolerance Distance tolerance (higher = more aggressive simplification)
 *                   Recommended: 0.00001-0.0001 for GPS coordinates
 * @returns Simplified array of points
 */
export function simplifyRoute<T extends Point>(
  points: T[],
  tolerance = 0.00005
): T[] {
  if (points.length <= 2) {
    return points;
  }

  const keep = new Array(points.length).fill(false);
  keep[0] = true;
  keep[points.length - 1] = true;

  douglasPeuckerRecursive(points, tolerance, 0, points.length - 1, keep);

  return points.filter((_, index) => keep[index]);
}

/**
 * Calculate the reduction percentage after simplification
 */
export function getSimplificationStats(
  originalCount: number,
  simplifiedCount: number
): { reduction: number; percentage: string } {
  const reduction = originalCount - simplifiedCount;
  const percentage = ((reduction / originalCount) * 100).toFixed(1);

  return { reduction, percentage };
}
