<script lang="ts">
  import { onMount } from "svelte";

  let status = $state<"success" | "error" | "loading">("loading");
  let provider = $state<string>("");
  let errorMessage = $state<string>("");

  onMount(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const urlStatus = urlParams.get("status");
    const urlProvider = urlParams.get("provider");
    const urlError = urlParams.get("error") || urlParams.get("message");

    if (urlStatus === "success" && urlProvider) {
      status = "success";
      provider = urlProvider;
      sendMessageToParent("success", urlProvider);
    } else if (urlStatus === "error") {
      status = "error";
      errorMessage =
        urlError || "An error occurred during OAuth connection";
      sendMessageToParent("error", urlProvider || "unknown", errorMessage);
    } else {
      status = "error";
      errorMessage = "Invalid callback parameters";
      sendMessageToParent("error", "unknown", errorMessage);
    }

    const timer = setTimeout(() => {
      window.close();
    }, 2000);

    return () => clearTimeout(timer);
  });

  function sendMessageToParent(
    messageStatus: "success" | "error",
    messageProvider: string,
    error?: string
  ) {
    if (!window.opener) {
      console.warn(
        "OAuth callback: window.opener is null, cannot send message to parent"
      );
      return;
    }

    const message = {
      type: "oauth-callback",
      status: messageStatus,
      provider: messageProvider,
      ...(error && { error }),
    };

    try {
      window.opener.postMessage(message, window.location.origin);
    } catch (err) {
      console.error("Failed to send message to parent window:", err);
    }
  }
</script>

<div
  class="min-h-screen flex items-center justify-center bg-gradient-to-br from-gray-50 to-gray-100"
>
  <div class="bg-white rounded-lg shadow-lg p-8 max-w-md w-full mx-4">
    {#if status === "loading"}
      <div class="text-center">
        <div
          class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"
        ></div>
        <h2 class="text-xl font-semibold text-gray-800">
          Processing...
        </h2>
      </div>
    {:else if status === "success"}
      <div class="text-center">
        <div
          class="bg-green-100 rounded-full p-3 w-16 h-16 mx-auto mb-4 flex items-center justify-center"
        >
          <svg
            class="w-10 h-10 text-green-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 13l4 4L19 7"
            ></path>
          </svg>
        </div>
        <h2 class="text-2xl font-bold text-gray-800 mb-2">
          Successfully connected!
        </h2>
        <p class="text-gray-600 mb-4">
          Your <span class="font-semibold capitalize">{provider}</span> account
          has been successfully connected.
        </p>
        <p class="text-sm text-gray-500">
          This window will close automatically...
        </p>
      </div>
    {:else}
      <div class="text-center">
        <div
          class="bg-red-100 rounded-full p-3 w-16 h-16 mx-auto mb-4 flex items-center justify-center"
        >
          <svg
            class="w-10 h-10 text-red-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            ></path>
          </svg>
        </div>
        <h2 class="text-2xl font-bold text-gray-800 mb-2">
          Connection failed
        </h2>
        <p class="text-gray-600 mb-4">{errorMessage}</p>
        <p class="text-sm text-gray-500">You can close this window.</p>
      </div>
    {/if}
  </div>
</div>
