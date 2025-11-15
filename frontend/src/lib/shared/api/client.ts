import { goto } from "$app/navigation";
import type { ApiError } from "./types";

class ApiClient {
  private baseUrl: string;

  constructor() {
    // En dev : localhost, en prod : variable d'environnement
    this.baseUrl = import.meta.env.VITE_API_URL || "http://localhost:3000";
  }

  /**
   * Requête HTTP générique avec gestion d'erreur
   */
  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;

    const config: RequestInit = {
      ...options,
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        ...options.headers,
      },
    };

    try {
      const response = await fetch(url, config);

      if (!response.ok) {
        await this.handleError(response);
      }

      return await response.json();
    } catch (error) {
      // Erreur réseau ou parsing
      console.error("API request failed:", error);
      throw error;
    }
  }

  /**
   * Gestion centralisée des erreurs
   */
  private async handleError(response: Response): Promise<never> {
    let errorData: ApiError;

    try {
      errorData = await response.json();
    } catch {
      // Si pas de JSON, créer une erreur générique
      errorData = {
        error: response.statusText,
        status: response.status,
      };
    }

    // Cas spécial : 401 = pas authentifié
    if (response.status === 401) {
      // Redirect vers login (sauf si on est déjà sur /auth/login)
      if (!window.location.pathname.startsWith("/auth")) {
        goto("/auth/login");
      }
    }

    // Throw l'erreur pour que le composant puisse la catch
    throw new Error(errorData.message || errorData.error);
  }

  /**
   * GET request
   */
  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, {
      method: "GET",
    });
  }

  /**
   * POST request
   */
  async post<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "POST",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  /**
   * PUT request
   */
  async put<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "PUT",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  /**
   * PATCH request
   */
  async patch<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "PATCH",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  /**
   * DELETE request
   */
  async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, {
      method: "DELETE",
    });
  }
}

// Export une instance unique (singleton)
export const apiClient = new ApiClient();
