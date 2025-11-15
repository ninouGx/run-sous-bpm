import { type } from "arktype";

export const loginSchema = type({
  email: "string",
  password: "string>8",
});

export const registerSchema = type({
  email: "string",
  password: "string>8",
  confirmPassword: "string>8",
}).narrow((data, ctx) => {
  if (data.password !== data.confirmPassword) {
    return ctx.mustBe("passwords matching");
  }
  return true;
});

export type LoginData = typeof loginSchema.infer;
export type RegisterData = typeof registerSchema.infer;
