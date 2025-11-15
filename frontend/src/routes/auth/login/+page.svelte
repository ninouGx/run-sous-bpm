<script lang="ts">
  import { goto } from "$app/navigation";
  import { authService } from "$lib/features/auth/auth.service";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  let email = $state("");
  let password = $state("");
  let isLoading = $state(false);
  let error = $state("");
  let validationErrors = $state<Record<string, string>>({});

  async function handleSubmit() {
    validationErrors = {};
    error = "";

    if (!email) {
      validationErrors.email = 'Email is required';
    }
    if (!password) {
      validationErrors.password = 'Password is required';
    } else if (password.length < 8) {
      validationErrors.password = 'Password must be at least 8 characters';
    }

    if (Object.keys(validationErrors).length > 0) {
      return;
    }

    isLoading = true;

    try {
      await authService.login({ email, password });
      goto("/dashboard");
    } catch (err) {
      error = err instanceof Error ? err.message : "Login failed";
    } finally {
      isLoading = false;
    }
  }
</script>

<Card>
  <CardHeader class="space-y-1">
    <CardTitle class="text-2xl font-bold">Login</CardTitle>
    <CardDescription
      >Enter your credentials to access your account</CardDescription
    >
  </CardHeader>
  <CardContent>
    <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="space-y-4">
      <div class="space-y-2">
        <Label for="email">Email</Label>
        <Input
          id="email"
          type="email"
          placeholder="you@example.com"
          bind:value={email}
          disabled={isLoading}
        />
        {#if validationErrors.email}
          <p class="text-sm text-destructive">{validationErrors.email}</p>
        {/if}
      </div>

      <div class="space-y-2">
        <Label for="password">Password</Label>
        <Input
          id="password"
          type="password"
          placeholder="••••••••"
          bind:value={password}
          disabled={isLoading}
        />
        {#if validationErrors.password}
          <p class="text-sm text-destructive">{validationErrors.password}</p>
        {/if}
      </div>

      {#if error}
        <div class="text-sm text-destructive bg-destructive/10 p-3 rounded-md">
          {error}
        </div>
      {/if}

      <Button type="submit" class="w-full" disabled={isLoading}>
        {isLoading ? "Logging in..." : "Login"}
      </Button>

      <p class="text-sm text-center text-muted-foreground">
        Don't have an account?
        <a href="/auth/register" class="text-primary hover:underline"
          >Register</a
        >
      </p>
    </form>
  </CardContent>
</Card>
