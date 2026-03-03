import React from "react";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import SymbolSearch from "./SymbolSearch";

const getWorkspaceSymbols = jest.fn();

jest.mock("@/lib/monaco-lsp", () => ({
  getLspClient: () => ({
    getWorkspaceSymbols,
  }),
  generateStableUri: (path: string) => `file:///workspace/${path}`,
}));

describe("SymbolSearch", () => {
  beforeEach(() => {
    jest.useFakeTimers();
    getWorkspaceSymbols.mockReset();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("renders workspace symbol results from the LSP client", async () => {
    getWorkspaceSymbols.mockResolvedValue([
      {
        name: "transfer",
        kind: 6,
        location: {
          uri: "file:///workspace/src/main.v",
          range: {
            start: { line: 2, character: 4 },
            end: { line: 2, character: 12 },
          },
        },
      },
    ]);

    render(<SymbolSearch isOpen={true} />);

    fireEvent.change(screen.getByPlaceholderText("Search symbols (Cmd+T)..."), {
      target: { value: "tran" },
    });

    await act(async () => {
      jest.advanceTimersByTime(250);
    });

    await waitFor(() => {
      expect(getWorkspaceSymbols).toHaveBeenCalledWith("tran");
    });

    expect(await screen.findByText("transfer")).toBeInTheDocument();
    expect(screen.getByText("src/main.v")).toBeInTheDocument();
  });
});
