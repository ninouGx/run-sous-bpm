<script lang="ts">
  import { userStore } from "$lib/stores/user";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { oauthService } from "$lib/features/auth/oauth.service";
  import { userService } from "$lib/features/auth/user.service";
  import { authService } from "$lib/features/auth/auth.service";
  import { OauthProvider } from "$lib/shared/api/types";
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";
  import { type } from "arktype";
  import { lastfmUsernameSchema } from "$lib/schemas/user";
  import { onMount, onDestroy } from "svelte";
  import {
    activitiesStore,
    sortedActivities,
    ActivityList,
  } from "$lib/features/activity";

  let stravaConnected = $derived(
    $userStore.user?.oauth_connections?.strava ?? false
  );
  let spotifyConnected = $derived(
    $userStore.user?.oauth_connections?.spotify ?? false
  );
  let isLastFmUsernameSet = $derived(!!$userStore.user?.lastfm_username);

  // Last.fm username editing state
  let isEditingLastfm = $state(false);
  let lastfmUsernameInput = $state("");
  let isLoadingLastfm = $state(false);
  let lastfmValidationError = $state<string | null>(null);

  async function handleConnect(service: OauthProvider) {
    let oauthUrl = await oauthService.getAuthorizationUrl(service);
    window.open(oauthUrl, "_blank", "width=600,height=700");
  }

  function handleDisconnect(service: OauthProvider) {
    oauthService
      .disconnectProvider(service)
      .then(() => {
        toast.success(`${service} disconnected successfully`);
      })
      .catch((error: unknown) => {
        const errorMessage =
          error instanceof Error
            ? error.message
            : `Failed to disconnect ${service}`;
        toast.error(errorMessage);
      });
  }

  function handleEditLastfm() {
    lastfmUsernameInput = $userStore.user?.lastfm_username ?? "";
    lastfmValidationError = null;
    isEditingLastfm = true;
  }

  function handleCancelLastfm() {
    isEditingLastfm = false;
    lastfmUsernameInput = "";
    lastfmValidationError = null;
  }

  async function handleSaveLastfm() {
    lastfmValidationError = null;

    // Client-side validation
    const validation = lastfmUsernameSchema(lastfmUsernameInput.trim());
    if (validation instanceof type.errors) {
      lastfmValidationError = validation.summary;
      return;
    }

    isLoadingLastfm = true;

    try {
      await userService.updateLastfmUsername(lastfmUsernameInput.trim());
      toast.success("Last.fm username updated successfully");
      isEditingLastfm = false;
      lastfmUsernameInput = "";
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : "Failed to update Last.fm username";

      // Show inline error for validation failures
      if (errorMessage.includes("Invalid Last.fm username")) {
        lastfmValidationError =
          "This Last.fm username does not exist. Please check and try again.";
      } else {
        lastfmValidationError = errorMessage;
      }

      // Also show toast for all errors
      toast.error(errorMessage);
    } finally {
      isLoadingLastfm = false;
    }
  }

  // OAuth popup message handler
  function handleOAuthMessage(event: MessageEvent) {
    // CRITICAL: Validate message origin for security
    if (event.origin !== window.location.origin) {
      console.warn(
        "Ignoring OAuth message from untrusted origin:",
        event.origin
      );
      return;
    }

    // Validate message structure
    const message = event.data;
    if (!message || message.type !== "oauth-callback") {
      return;
    }

    // Handle OAuth callback result
    if (message.status === "success") {
      const providerName =
        message.provider.charAt(0).toUpperCase() + message.provider.slice(1);
      toast.success(`${providerName} connected successfully!`);

      // Refresh user state to get updated OAuth connections
      authService.checkAuth();
    } else if (message.status === "error") {
      const errorMessage = message.error || "Failed to connect OAuth provider";
      toast.error(errorMessage);
    }
  }

  // Load activities when component mounts
  async function loadActivities() {
    if (stravaConnected) {
      await activitiesStore.load();
    }
  }

  // Handle sync
  async function handleSyncActivities() {
    try {
      await activitiesStore.sync();
      toast.success("Activities synced successfully!");
    } catch (error: unknown) {
      toast.error(
        error instanceof Error ? error.message : "Failed to sync activities"
      );
    }
  }

  // Setup and cleanup message listener
  onMount(() => {
    window.addEventListener("message", handleOAuthMessage);
    loadActivities();
  });

  onDestroy(() => {
    window.removeEventListener("message", handleOAuthMessage);
  });
</script>

<div class="space-y-6">
  <!-- Header -->
  <div>
    <h1 class="text-3xl font-bold">Dashboard</h1>
    <p class="text-muted-foreground">Welcome back, {$userStore.user?.email}</p>
  </div>

  <!-- Main Content: Activities (left) + Sidebar (right) -->
  <div class="flex flex-col lg:flex-row gap-8">
    <!-- Main Content: Activities List -->
    <div class="flex-1">
      {#if stravaConnected}
        <ActivityList
          activities={$sortedActivities}
          isLoading={$activitiesStore.isLoading}
          error={$activitiesStore.error}
          onSync={handleSyncActivities}
        />
      {:else}
        <Card>
          <CardContent class="py-16">
            <div class="text-center">
              <h3 class="text-lg font-semibold mb-2">
                Connect Strava to Get Started
              </h3>
              <p class="text-sm text-muted-foreground mb-4">
                Connect your Strava account to view and sync your activities
              </p>
              <Button onclick={() => handleConnect(OauthProvider.Strava)}>
                Connect Strava
              </Button>
            </div>
          </CardContent>
        </Card>
      {/if}
    </div>

    <!-- Sidebar: Connected Services -->
    <aside class="lg:w-80 space-y-4">
      <h2 class="text-xl font-semibold mb-4">Connected Services</h2>
      <div class="flex flex-col gap-2">
        <Card>
          <CardHeader>
            <div class="flex items-center justify-between">
              <CardTitle class="text-lg">Strava</CardTitle>
              <Badge variant={stravaConnected ? "default" : "secondary"}>
                {stravaConnected ? "Connected" : "Disconnected"}
              </Badge>
            </div>
          </CardHeader>
          <CardContent>
            {#if stravaConnected}
              <Button
                variant="outline"
                size="sm"
                class="w-full"
                onclick={() => handleDisconnect(OauthProvider.Strava)}
              >
                Disconnect
              </Button>
            {:else}
              <Button
                size="sm"
                class="w-full"
                onclick={() => handleConnect(OauthProvider.Strava)}
              >
                Connect
              </Button>
            {/if}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div class="flex items-center justify-between">
              <CardTitle class="text-lg">Spotify</CardTitle>
              <Badge variant={spotifyConnected ? "default" : "secondary"}>
                {spotifyConnected ? "Connected" : "Disconnected"}
              </Badge>
            </div>
          </CardHeader>
          <CardContent>
            {#if spotifyConnected}
              <Button
                variant="outline"
                size="sm"
                class="w-full"
                onclick={() => handleDisconnect(OauthProvider.Spotify)}
              >
                Disconnect
              </Button>
            {:else}
              <Button
                size="sm"
                class="w-full"
                onclick={() => handleConnect(OauthProvider.Spotify)}
              >
                Connect
              </Button>
            {/if}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div class="flex items-center justify-between">
              <CardTitle class="text-lg">Last.fm</CardTitle>
              <Badge variant={isLastFmUsernameSet ? "default" : "secondary"}>
                {isLastFmUsernameSet ? "Connected" : "Disconnected"}
              </Badge>
            </div>
          </CardHeader>
          <CardContent>
            {#if isEditingLastfm}
              <div class="space-y-3">
                <div class="space-y-2">
                  <Label for="lastfm-username">Last.fm Username</Label>
                  <Input
                    id="lastfm-username"
                    type="text"
                    placeholder="Enter your Last.fm username"
                    bind:value={lastfmUsernameInput}
                    disabled={isLoadingLastfm}
                    class={lastfmValidationError ? "border-destructive" : ""}
                  />
                  {#if lastfmValidationError}
                    <p class="text-sm text-destructive">
                      {lastfmValidationError}
                    </p>
                  {/if}
                </div>
                <div class="flex gap-2">
                  <Button
                    size="sm"
                    class="flex-1"
                    onclick={handleSaveLastfm}
                    disabled={isLoadingLastfm}
                  >
                    {isLoadingLastfm ? "Saving..." : "Save"}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    class="flex-1"
                    onclick={handleCancelLastfm}
                    disabled={isLoadingLastfm}
                  >
                    Cancel
                  </Button>
                </div>
              </div>
            {:else if isLastFmUsernameSet}
              <div class="space-y-2">
                <p class="text-sm text-muted-foreground">
                  Username: <span class="font-medium text-foreground"
                    >{$userStore.user?.lastfm_username}</span
                  >
                </p>
                <Button
                  variant="outline"
                  size="sm"
                  class="w-full"
                  onclick={handleEditLastfm}
                >
                  Change Username
                </Button>
              </div>
            {:else}
              <Button size="sm" class="w-full" onclick={handleEditLastfm}>
                Set Username
              </Button>
            {/if}
          </CardContent>
        </Card>
      </div>
    </aside>
  </div>
</div>
