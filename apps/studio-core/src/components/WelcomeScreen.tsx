import { useState, useEffect } from 'react';
import { X, Zap, Code, Sparkles } from 'lucide-react';
import { type ExampleKey } from '../examples';
import './WelcomeScreen.css';

interface WelcomeScreenProps {
    onClose: () => void;
    onSelectExample: (key: ExampleKey) => void;
}

const templates = [
    {
        key: 'Simple Web App' as ExampleKey,
        title: 'Simple Web App',
        description: 'Start from a basic app',
        icon: Zap,
        color: 'var(--color-info-500)',
    },
    {
        key: 'Microservices Architecture' as ExampleKey,
        title: 'Microservices',
        description: 'Services and queues',
        icon: Code,
        color: 'var(--color-primary-500)',
    },
    {
        key: 'E-Commerce Platform' as ExampleKey,
        title: 'E‑Commerce',
        description: 'Shop, payments, orders',
        icon: Sparkles,
        color: 'var(--color-brand-pink)',
    },
];

export function WelcomeScreen({ onClose, onSelectExample }: WelcomeScreenProps) {
    const [dontShowAgain, setDontShowAgain] = useState(false);
    const [isClosing, setIsClosing] = useState(false);

    const handleClose = () => {
        if (dontShowAgain) {
            localStorage.setItem('studio-hide-welcome', 'true');
        }
        setIsClosing(true);
        setTimeout(onClose, 300);
    };

    const handleSelectTemplate = (key: ExampleKey) => {
        if (dontShowAgain) {
            localStorage.setItem('studio-hide-welcome', 'true');
        }
        onSelectExample(key);
        setIsClosing(true);
        setTimeout(onClose, 300);
    };

    // Prevent scroll when modal is open
    useEffect(() => {
        document.body.style.overflow = 'hidden';
        return () => {
            document.body.style.overflow = '';
        };
    }, []);

    return (
        <div className={`welcome-overlay ${isClosing ? 'closing' : ''}`}>
            <div className={`welcome-modal ${isClosing ? 'modal-closing' : ''}`}>
                {/* Close Button */}
                <button
                    onClick={handleClose}
                    className="welcome-close-btn"
                    aria-label="Close welcome screen"
                >
                    <X size={20} />
                </button>

                {/* Header */}
                <div className="welcome-header">
                    <div className="welcome-logo">
                        <svg width="60" height="60" viewBox="0 0 100 100">
                            <defs>
                                <linearGradient id="welcomeGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                                    <stop offset="0%" style={{ stopColor: '#667eea' }} />
                                    <stop offset="100%" style={{ stopColor: '#764ba2' }} />
                                </linearGradient>
                            </defs>
                            <circle cx="50" cy="50" r="40" fill="none" stroke="url(#welcomeGradient)" strokeWidth="4" />
                            <path d="M 50 20 L 50 50 L 70 70" fill="none" stroke="url(#welcomeGradient)" strokeWidth="4" strokeLinecap="round" />
                        </svg>
                    </div>
                    <h1 className="welcome-title">Sruja Studio</h1>
                    <p className="welcome-subtitle">Pick a template to start</p>
                </div>

                {/* Templates */}
                <div className="welcome-section">
                    <h2 className="section-title">Templates</h2>
                    <div className="template-grid">
                        {templates.map((template) => {
                            const Icon = template.icon;
                            return (
                                <button
                                    key={template.key}
                                    onClick={() => handleSelectTemplate(template.key)}
                                    className="template-card"
                                >
                                    <div className="template-icon" style={{ backgroundColor: template.color }}>
                                        <Icon size={28} color="white" />
                                    </div>
                                    <h3 className="template-title">{template.title}</h3>
                                    <p className="template-description">{template.description}</p>
                                </button>
                            );
                        })}
                    </div>
                </div>

                {/* Footer */}
                <div className="welcome-footer">
                    <label className="checkbox-label">
                        <input
                            type="checkbox"
                            checked={dontShowAgain}
                            onChange={(e) => setDontShowAgain(e.target.checked)}
                        />
                        <span>Don't show this again</span>
                    </label>
                    <button onClick={handleClose} className="btn-primary">
                        Get Started
                    </button>
                </div>
            </div>
        </div>
    );
}
