let fromABIImpl: (...args: any[]) => any = () => {
  throw new Error("FiveProgram.fromABI mock not initialized");
};

export function __setFromABIImpl(impl: (...args: any[]) => any): void {
  fromABIImpl = impl;
}

export const FiveProgram = {
  fromABI: (...args: any[]) => fromABIImpl(...args),
};
