import { ChevronRight, Home } from 'lucide-react';
import { useViewStore } from '../../stores';
import './Breadcrumb.css';

export function Breadcrumb() {
    const breadcrumb = useViewStore((s) => s.breadcrumb);
    const goToRoot = useViewStore((s) => s.goToRoot);
    const goUp = useViewStore((s) => s.goUp);

    if (!breadcrumb || !Array.isArray(breadcrumb)) {
        return null;
    }

    return (
        <nav className="breadcrumb">
            <button className="breadcrumb-item home" onClick={goToRoot} title="Go to root">
                <Home size={16} />
            </button>
            {breadcrumb.map((item, index) => (
                <span key={index} className="breadcrumb-segment">
                    <ChevronRight size={14} className="breadcrumb-separator" />
                    <button
                        className={`breadcrumb-item ${index === breadcrumb.length - 1 ? 'current' : ''}`}
                        onClick={() => {
                            // Navigate to this level
                            const stepsBack = breadcrumb.length - 1 - index;
                            for (let i = 0; i < stepsBack; i++) {
                                goUp();
                            }
                        }}
                    >
                        {item}
                    </button>
                </span>
            ))}
        </nav>
    );
}
