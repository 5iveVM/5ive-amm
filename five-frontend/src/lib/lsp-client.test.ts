import { FiveLspClient } from "./lsp-client";

describe("FiveLspClient workspace aggregation", () => {
  function createClient() {
    const client = new FiveLspClient();
    const clientAny = client as any;
    clientAny.initialized = true;
    clientAny.lsp = {
      get_workspace_symbols: jest.fn(),
      get_diagnostics: jest.fn(),
      clear_caches: jest.fn(),
    };
    return {
      client,
      lsp: clientAny.lsp,
    };
  }

  it("tracks documents and aggregates workspace symbols across files", async () => {
    const { client, lsp } = createClient();

    lsp.get_workspace_symbols.mockImplementation(async (uri: string) => {
      if (uri.endsWith("alpha.v")) {
        return JSON.stringify([
          {
            name: "alpha",
            kind: 6,
            location: {
              uri,
              range: { start: { line: 1, character: 0 }, end: { line: 1, character: 5 } },
            },
          },
          {
            name: "alpha",
            kind: 6,
            location: {
              uri,
              range: { start: { line: 1, character: 0 }, end: { line: 1, character: 5 } },
            },
          },
        ]);
      }

      return JSON.stringify([
        {
          name: "beta",
          kind: 6,
          location: {
            uri,
            range: { start: { line: 0, character: 0 }, end: { line: 0, character: 4 } },
          },
        },
      ]);
    });

    await client.setDocument("file:///workspace/alpha.v", "alpha() -> u64 { return 1; }");
    await client.setDocument("file:///workspace/beta.v", "beta() -> u64 { return 2; }");

    const symbols = await client.getWorkspaceSymbols("a");

    expect(lsp.get_workspace_symbols).toHaveBeenCalledTimes(2);
    expect(symbols.map((symbol) => symbol.name)).toEqual(["alpha", "beta"]);
    expect(symbols[0].location.uri).toBe("file:///workspace/alpha.v");
    expect(symbols[1].location.uri).toBe("file:///workspace/beta.v");
  });

  it("fans out diagnostics to every tracked document and removes disposed documents", async () => {
    const { client, lsp } = createClient();

    lsp.get_diagnostics.mockImplementation((uri: string) =>
      JSON.stringify([
        {
          message: `diag:${uri}`,
          range: {
            start: { line: 0, character: 0 },
            end: { line: 0, character: 1 },
          },
          severity: 1,
        },
      ])
    );

    await client.setDocument("file:///workspace/one.v", "one");
    await client.setDocument("file:///workspace/two.v", "two");

    const beforeRemove = await client.getWorkspaceDiagnostics();
    expect(beforeRemove.size).toBe(2);
    expect(beforeRemove.get("file:///workspace/one.v")?.[0].message).toBe(
      "diag:file:///workspace/one.v"
    );

    await client.removeDocument("file:///workspace/one.v");

    const afterRemove = await client.getWorkspaceDiagnostics();
    expect(afterRemove.size).toBe(1);
    expect(afterRemove.has("file:///workspace/one.v")).toBe(false);
    expect(afterRemove.has("file:///workspace/two.v")).toBe(true);
  });
});
