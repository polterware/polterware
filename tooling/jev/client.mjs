import { TypeSafeClient } from "@typesafe-ai/sdk";

export function createClient() {
  if (!process.env.TYPESAFE_API_KEY?.trim()) {
    throw new Error("TYPESAFE_API_KEY is missing. Set it in the repository .env.");
  }
  return new TypeSafeClient();
}
