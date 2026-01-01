export const ROSE_PINE_DARK = {
    base: 'vs-dark',
    inherit: true,
    rules: [
        { token: "keyword", foreground: "eb6f92", fontStyle: "bold" },
        { token: "keyword.decl", foreground: "31748f", fontStyle: "bold" },
        { token: "annotation", foreground: "c4a7e7", fontStyle: "italic" },
        { token: "type", foreground: "9ccfd8" },
        { token: "function", foreground: "f6c177" },
        { token: "identifier", foreground: "e0def4" },
        { token: "comment", foreground: "6e6a86", fontStyle: "italic" },
        { token: "string", foreground: "ebbcba" },
        { token: "number", foreground: "c4a7e7" },
    ],
    colors: {
        "editor.background": "#2a273f00", // Transparent
        "editor.foreground": "#e0def4",
        "editor.lineHighlightBackground": "#44415a40",
        "editorLineNumber.foreground": "#908caa",
        "editorCursor.foreground": "#e0def4",
        "editor.selectionBackground": "#44415a80",
        "scrollbar.shadow": "#00000000",
    },
};

export const ROSE_PINE_LIGHT = {
    base: 'vs',
    inherit: true,
    rules: [
        { token: "keyword", foreground: "b4637a", fontStyle: "bold" },
        { token: "keyword.decl", foreground: "286983", fontStyle: "bold" },
        { token: "annotation", foreground: "907aa9", fontStyle: "italic" },
        { token: "type", foreground: "56949f" },
        { token: "function", foreground: "ea9d34" },
        { token: "identifier", foreground: "575279" },
        { token: "comment", foreground: "9893a5", fontStyle: "italic" },
        { token: "string", foreground: "d7827e" },
        { token: "number", foreground: "907aa9" },
    ],
    colors: {
        "editor.background": "#faf4ed00", // Transparent
        "editor.foreground": "#575279",
        "editor.lineHighlightBackground": "#dfdad940",
        "editorLineNumber.foreground": "#797593",
        "editorCursor.foreground": "#575279",
        "editor.selectionBackground": "#dfdad980",
        "scrollbar.shadow": "#00000000",
    },
};

export const registerFiveLanguage = (monaco: any) => {
    // Only register if not already registered
    if (monaco.languages.getLanguages().some((l: any) => l.id === 'five')) {
        return;
    }

    monaco.languages.register({ id: "five" });

    monaco.languages.setMonarchTokensProvider("five", {
        tokenizer: {
            root: [
                [/\b(function|let|if|else|return|const|import|from)\b/, "keyword"],
                [/\b(struct|event|emit|account|pub|init)\b/, "keyword.decl"],
                [/@[a-zA-Z_]\w*/, "annotation"],
                [/\b(u64|u8|bool|pubkey|string|array)\b/, "type"],
                [/[a-zA-Z_]\w*(?=\s*\()/, "function"],
                [/[a-zA-Z_]\w*/, "identifier"],
                [/\/\/.*$/, "comment"],
                [/"[^"]*"/, "string"],
                [/\d+/, "number"],
            ],
        },
    });

    monaco.languages.setLanguageConfiguration("five", {
        comments: { lineComment: "//" },
        brackets: [["{", "}"], ["[", "]"], ["(", ")"]],
    });
};

export const defineMonacoThemes = (monaco: any) => {
    // Define themes (safe to call multiple times as they overwrite)
    monaco.editor.defineTheme("rose-pine-dark", ROSE_PINE_DARK);
    monaco.editor.defineTheme("rose-pine-light", ROSE_PINE_LIGHT);
};
