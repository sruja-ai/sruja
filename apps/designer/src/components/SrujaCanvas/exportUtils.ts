

// Using html-to-image might be better than html2canvas for modern React apps, 
// checking deps... wait, user plan said html2canvas. Let's stick to html2canvas if available, OR use a simpler approach.
// Ah, package.json had html2canvas. Let's use that.

import html2canvas from 'html2canvas';
import { toSvg } from 'html-to-image';

export const downloadImage = () => {
    // ... existing png logic ...
    // Simplest approach: Capture current view from .react-flow container
    const element = document.querySelector('.react-flow') as HTMLElement;
    if (!element) return;

    html2canvas(element, {
        backgroundColor: '#ffffff',
        ignoreElements: (element) => {
            const className = element.className;
            if (typeof className === 'string') {
                return className.includes('react-flow__controls') ||
                    className.includes('react-flow__minimap') ||
                    className.includes('mantine');
            }
            return false;
        }
    }).then((canvas) => {
        const dataUrl = canvas.toDataURL('image/png');
        downloadDataUrl(dataUrl, 'sruja-diagram.png');
    });
};

export const downloadSvg = async () => {
    const element = document.querySelector('.react-flow') as HTMLElement;
    if (!element) return;

    try {
        const dataUrl = await toSvg(element, {
            backgroundColor: '#ffffff',
            filter: (node) => {
                const className = (node as HTMLElement).className;
                if (typeof className === 'string') {
                    return !className.includes('react-flow__controls') &&
                        !className.includes('react-flow__minimap') &&
                        !className.includes('mantine');
                }
                return true;
            }
        });
        downloadDataUrl(dataUrl, 'sruja-diagram.svg');
    } catch (err) {
        console.error('Error exporting SVG:', err);
    }
};

function downloadDataUrl(dataUrl: string, filename: string) {
    const a = document.createElement('a');
    a.href = dataUrl;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
}
