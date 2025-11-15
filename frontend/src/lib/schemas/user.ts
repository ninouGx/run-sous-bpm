import { type } from "arktype";

// Last.fm username validation
// - Must be non-empty
// - 2-30 characters
// - Letters, numbers, hyphens, and underscores only
export const lastfmUsernameSchema = type("string").narrow((username, ctx) => {
  if (username.length < 2 || username.length > 30) {
    return ctx.mustBe("between 2 and 30 characters");
  }
  if (!/^[a-zA-Z0-9_-]+$/.test(username)) {
    return ctx.mustBe("alphanumeric characters, hyphens, or underscores only");
  }
  return true;
});

export type LastfmUsername = typeof lastfmUsernameSchema.infer;
