import { execFile } from 'child_process';
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const serverPath = await resolveServerPath();
  if (!serverPath) {
    return;
  }

  const serverOptions: ServerOptions = {
    run:   { command: serverPath, transport: TransportKind.stdio },
    debug: { command: serverPath, transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'anti-gravital' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.ag'),
    },
  };

  client = new LanguageClient(
    'anti-gravital-lsp',
    'Anti-Gravital LSP',
    serverOptions,
    clientOptions,
  );

  context.subscriptions.push(client);
  await client.start();
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
  }
}

async function resolveServerPath(): Promise<string | undefined> {
  const found = await findInPath('ag-lsp');
  if (found) {
    return found;
  }

  const choice = await vscode.window.showInformationMessage(
    'ag-lsp not found. Install it with cargo to enable IntelliSense for .ag files.',
    'Install via cargo',
    'Dismiss',
  );

  if (choice !== 'Install via cargo') {
    return undefined;
  }

  const terminal = vscode.window.createTerminal('Install ag-lsp');
  terminal.show();
  terminal.sendText('cargo install ag-lsp');

  const reload = await vscode.window.showInformationMessage(
    'Installing ag-lsp in the terminal. Reload the window after installation completes.',
    'Reload Window',
  );
  if (reload === 'Reload Window') {
    await vscode.commands.executeCommand('workbench.action.reloadWindow');
  }

  return undefined;
}

// Usa execFile (no exec) para evitar interpretacion de shell.
// El argumento binary es siempre una constante interna ('ag-lsp').
function findInPath(binary: string): Promise<string | undefined> {
  const cmd = process.platform === 'win32' ? 'where' : 'which';
  return new Promise((resolve) => {
    execFile(cmd, [binary], (error, stdout) => {
      if (error || !stdout.trim()) {
        resolve(undefined);
      } else {
        resolve(stdout.trim().split('\n')[0].trim());
      }
    });
  });
}
