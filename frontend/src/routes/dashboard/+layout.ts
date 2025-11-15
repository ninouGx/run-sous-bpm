import { redirect } from "@sveltejs/kit";
import { authService } from "$lib/features/auth/auth.service";
import type { LayoutLoad } from "./$types";

export const ssr = false; // Force CSR pour le dashboard

export const load: LayoutLoad = async () => {
  // Vérifier l'authentification
  const user = await authService.checkAuth();

  if (!user) {
    // Pas authentifié → Redirect vers login
    throw redirect(303, "/auth/login");
  }

  return {
    user,
  };
};
