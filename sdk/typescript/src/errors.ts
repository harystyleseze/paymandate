/**
 * Typed mapping of the contract's error codes
 * (contracts/mandate/src/lib.rs `Error` enum).
 */
export class PayMandateError extends Error {
  constructor(
    public readonly code: number,
    public readonly name_: string,
    message: string,
  ) {
    super(message);
    this.name = "PayMandateError";
  }
}

const ERRORS: Record<number, [string, string]> = {
  1: ["MandateNotFound", "No mandate exists with this id."],
  2: [
    "MandateNotActive",
    "The mandate was revoked by the payer. Stop charging it and update your records.",
  ],
  3: [
    "MandateExpired",
    "The mandate's expiry has passed. Ask the customer to create a new mandate.",
  ],
  4: [
    "ExceedsPeriodCap",
    "This charge would exceed the per-period cap. Wait for the next period or charge a smaller amount.",
  ],
  5: [
    "InvalidParams",
    "Invalid parameters: amount and period must be positive, expiry must be in the future.",
  ],
};

/** Convert a contract error code into a typed, explained error. */
export function errorFromCode(code: number): PayMandateError {
  const entry = ERRORS[code];
  if (!entry) {
    return new PayMandateError(code, "Unknown", `Unknown contract error code ${code}.`);
  }
  return new PayMandateError(code, entry[0], entry[1]);
}
