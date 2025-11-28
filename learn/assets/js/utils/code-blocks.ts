// Code block enhancement utilities
import { initSrujaWasm, compileSrujaCode } from './wasm';

function slugify(s: string): string {
  return s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '') || 'snippet';
}

export function enhanceSrujaBlocks(): void {
  const blocks = document.querySelectorAll('pre > code.language-sruja');
  const pageSlug = slugify(window.location.pathname.replace(/\//g, '-'));
  blocks.forEach((code, idx) => {
    const pre = code.parentElement;
    if (!pre || (pre as HTMLElement).dataset.enhanced === 'true') return;
    (pre as HTMLElement).dataset.enhanced = 'true';

    const wrapper = document.createElement('div');
    wrapper.className = 'sruja-code-wrapper';

    const toolbar = document.createElement('div');
    toolbar.className = 'sruja-code-toolbar';

    const btnCopy = document.createElement('button');
    btnCopy.className = 'sruja-btn sruja-btn-copy';
    btnCopy.title = 'Copy';
    btnCopy.innerHTML = '📋';

    const btnEdit = document.createElement('button');
    btnEdit.className = 'sruja-btn sruja-btn-edit';
    btnEdit.title = 'Edit';
    btnEdit.innerHTML = '✏️';

    const btnRun = document.createElement('button');
    btnRun.className = 'sruja-btn sruja-btn-run';
    btnRun.title = 'Run';
    btnRun.innerHTML = '▶';

    const output = document.createElement('div');
    output.className = 'sruja-run-output';
    output.style.display = 'none';

    pre.parentNode?.insertBefore(wrapper, pre);
    wrapper.appendChild(pre);
    wrapper.appendChild(toolbar);
    toolbar.appendChild(btnCopy);
    toolbar.appendChild(btnEdit);
    toolbar.appendChild(btnRun);
    wrapper.appendChild(output);

    let editor: HTMLTextAreaElement | null = null;
    const currentSource = () => (editor ? editor.value : code.textContent || '');

    btnCopy.addEventListener('click', () => {
      const text = currentSource();
      navigator.clipboard && navigator.clipboard.writeText(text);
      btnCopy.classList.add('success');
      setTimeout(() => btnCopy.classList.remove('success'), 1200);
    });

    btnEdit.addEventListener('click', () => {
      if (!editor) {
        editor = document.createElement('textarea');
        editor.className = 'sruja-editor';
        editor.value = code.textContent || '';
        (pre as HTMLElement).style.display = 'none';
        wrapper.insertBefore(editor, toolbar.nextSibling);
        btnEdit.classList.add('active');
      } else {
        code.textContent = editor.value;
        editor.remove();
        editor = null;
        (pre as HTMLElement).style.display = '';
        btnEdit.classList.remove('active');
      }
    });

    btnRun.addEventListener('click', () => {
      initSrujaWasm();
      if (!window.srujaWasmReady || typeof window.compileSruja === 'undefined') {
        output.style.display = 'block';
        output.innerText = 'WASM not ready';
        return;
      }
      output.style.display = 'block';
      output.innerHTML = '';
      try {
        const filename = `page-${pageSlug}-snippet-${idx}.sruja`;
        const result = compileSrujaCode(currentSource(), filename);
        if (result?.error) {
          output.innerText = result.error;
        } else if (result?.svg) {
          output.innerHTML = result.svg;
        } else {
          output.innerText = 'No output';
        }
      } catch (e) {
        output.innerText = 'Internal Error: ' + (e instanceof Error ? e.message : String(e));
      }
    });

    btnRun.disabled = false;
    btnRun.title = 'Run';
  });
}

