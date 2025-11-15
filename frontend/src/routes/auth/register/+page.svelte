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
  let confirmPassword = $state("");
  let isLoading = $state(false);
  let error = $state("");
  let success = $state(false);
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
    if (!confirmPassword) {
      validationErrors.confirmPassword = 'Please confirm your password';
    } else if (password !== confirmPassword) {
      validationErrors.form = 'Passwords do not match';
    }

    if (Object.keys(validationErrors).length > 0) {
      return;
    }

    isLoading = true;

    try {
      await authService.register({ email, password });
      success = true;

      setTimeout(() => {
        goto("/auth/login");
      }, 2000);
    } catch (err) {
      error = err instanceof Error ? err.message : "Registration failed";
    } finally {
      isLoading = false;
    }
  }
</script>

<Card>
  <CardHeader class="space-y-1">
    <CardTitle class="text-2xl font-bold">Create an account</CardTitle>
    <CardDescription>Enter your information to get started</CardDescription>
  </CardHeader>
  <CardContent>
    {#if success}
      <div
        class="text-sm bg-primary/10 text-primary p-4 rounded-md text-center"
      >
        <p class="font-semibold">Account created successfully!</p>
        <p class="mt-1">Redirecting to login...</p>
      </div>
    {:else}
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
          {:else}
            <p class="text-xs text-muted-foreground">
              Must be at least 8 characters
            </p>
          {/if}
        </div>

        <div class="space-y-2">
          <Label for="confirm-password">Confirm Password</Label>
          <Input
            id="confirm-password"
            type="password"
            placeholder="••••••••"
            bind:value={confirmPassword}
            disabled={isLoading}
          />
          {#if validationErrors.confirmPassword}
            <p class="text-sm text-destructive">{validationErrors.confirmPassword}</p>
          {/if}
        </div>

        {#if error}
          <div class="text-sm text-destructive bg-destructive/10 p-3 rounded-md">
            {error}
          </div>
        {/if}
        {#if validationErrors.form}
          <div class="text-sm text-destructive bg-destructive/10 p-3 rounded-md">
            {validationErrors.form}
          </div>
        {/if}

        <Button type="submit" class="w-full" disabled={isLoading}>
          {isLoading ? "Creating account..." : "Create account"}
        </Button>

        <p class="text-sm text-center text-muted-foreground">
          Already have an account?
          <a href="/auth/login" class="text-primary hover:underline">Login</a>
        </p>
      </form>
    {/if}
  </CardContent>
</Card>
