type VMInfo = {
  version: string;
  features: string[];
};

export class FiveVM {
  constructor(_logger?: any) {}

  async initialize(): Promise<void> {}

  getVMInfo(): VMInfo {
    return { version: 'mock', features: [] };
  }
}
