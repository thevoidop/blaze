const vscode = require("vscode");

function activate(context) {
  const disposable = vscode.languages.registerDocumentFormattingEditProvider(
    "blaze",
    {
      provideDocumentFormattingEdits(document) {
        const text = document.getText();
        const formatted = formatBlazeCode(text);
        const range = new vscode.Range(
          document.positionAt(0),
          document.positionAt(text.length),
        );
        return [vscode.TextEdit.replace(range, formatted)];
      },
    },
  );

  context.subscriptions.push(disposable);
}

function formatBlazeCode(code) {
  const lines = code.split(/\r?\n/);
  let indentLevel = 0;
  const indentStr = "    "; // 4 spaces per indent level
  const formattedLines = [];

  for (let line of lines) {
    let trimmed = line.trim();

    // Decrease indent level if line starts with a closing brace
    if (trimmed.startsWith("}")) {
      indentLevel = Math.max(0, indentLevel - 1);
    }

    if (trimmed === "") {
      formattedLines.push("");
      continue;
    }

    // Apply indentation
    formattedLines.push(indentStr.repeat(indentLevel) + trimmed);

    // Increase indent level if line ends with an opening brace
    if (trimmed.endsWith("{")) {
      indentLevel++;
    }
  }

  // Cleanly trim trailing blank lines and ensure exactly one newline at EOF
  return formattedLines.join("\n").trimEnd() + "\n";
}

function deactivate() {}

module.exports = {
  activate,
  deactivate,
};
