import * as vscode from "vscode";
import { ExtensionContext } from "../__mocks__/vscode";

import { registerCommandCenter } from "./commandCenter";

describe("registerCommandCenter", () => {
  let registered: Map<string, () => Promise<void> | void>;

  beforeEach(() => {
    registered = new Map<string, () => Promise<void> | void>();
    (vscode.commands as any).registerCommand = jest.fn((id: string, cb: () => Promise<void> | void) => {
      registered.set(id, cb);
      return { dispose: () => {} };
    });
    (vscode.commands as any).executeCommand = jest.fn();
    (vscode.window as any).showQuickPick = jest.fn();
  });

  it("shows a quick pick and executes the selected command", async () => {
    (vscode.window as any).showQuickPick.mockResolvedValue({
      label: "Validate",
      description: "",
      command: "sruja.runValidation",
    });

    registerCommandCenter(new ExtensionContext() as any);
    const cb = registered.get("sruja.commandCenter");
    if (!cb) throw new Error("Command not registered: sruja.commandCenter");

    await cb();

    expect(vscode.window.showQuickPick).toHaveBeenCalled();
    expect(vscode.commands.executeCommand).toHaveBeenCalledWith("sruja.runValidation");
  });

  it("does nothing when the user cancels", async () => {
    (vscode.window as any).showQuickPick.mockResolvedValue(undefined);

    registerCommandCenter(new ExtensionContext() as any);
    const cb = registered.get("sruja.commandCenter");
    if (!cb) throw new Error("Command not registered: sruja.commandCenter");

    await cb();

    expect(vscode.commands.executeCommand).not.toHaveBeenCalled();
  });
});
