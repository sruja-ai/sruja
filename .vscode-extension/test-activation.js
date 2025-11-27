// Simple test to check if extension can activate
// Run this in Trae's developer console to test

const vscode = require('vscode');

console.log('Testing extension activation...');

// Try to register a simple command
const testCommand = vscode.commands.registerCommand('sruja.test', () => {
  console.log('Test command executed!');
  vscode.window.showInformationMessage('Extension is working!');
});

console.log('Test command registered:', testCommand !== undefined);

