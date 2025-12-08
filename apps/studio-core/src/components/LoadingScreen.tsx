import { useEffect, useState } from 'react';
import './LoadingScreen.css';

interface LoadingScreenProps {
    progress?: number;
    message?: string;
    isLoading: boolean;
}

export function LoadingScreen({ progress, message = 'Initializing Sruja Studio...', isLoading }: LoadingScreenProps) {
    const [show, setShow] = useState(isLoading);
    const [fadeOut, setFadeOut] = useState(false);

    useEffect(() => {
        if (!isLoading) {
            setFadeOut(true);
            const timer = setTimeout(() => setShow(false), 500);
            return () => clearTimeout(timer);
        } else {
            setShow(true);
            setFadeOut(false);
        }
    }, [isLoading]);

    if (!show) return null;

    return (
        <div className={`loading-screen ${fadeOut ? 'fade-out' : ''}`}>
            <div className="loading-content">
                {/* Animated Logo */}
                <div className="loading-logo">
                    <svg
                        width="80"
                        height="80"
                        viewBox="0 0 100 100"
                        className="logo-animate"
                    >
                        <defs>
                            <linearGradient id="logoGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                                <stop offset="0%" style={{ stopColor: '#667eea', stopOpacity: 1 }} />
                                <stop offset="100%" style={{ stopColor: '#764ba2', stopOpacity: 1 }} />
                            </linearGradient>
                        </defs>
                        <circle
                            cx="50"
                            cy="50"
                            r="40"
                            fill="none"
                            stroke="url(#logoGradient)"
                            strokeWidth="4"
                            strokeLinecap="round"
                            className="logo-circle"
                        />
                        <path
                            d="M 50 20 L 50 50 L 70 70"
                            fill="none"
                            stroke="url(#logoGradient)"
                            strokeWidth="4"
                            strokeLinecap="round"
                            className="logo-path"
                        />
                    </svg>
                </div>

                {/* App Title */}
                <h1 className="loading-title">Sruja Studio</h1>
                <p className="loading-subtitle">Architecture Diagramming Made Simple</p>

                {/* Progress Indicator */}
                {progress !== undefined ? (
                    <div className="loading-progress">
                        <div className="progress-bar">
                            <div
                                className="progress-bar-fill"
                                style={{ width: `${progress}%` }}
                            />
                        </div>
                        <span className="progress-text">{Math.round(progress)}%</span>
                    </div>
                ) : (
                    <div className="loading-progress">
                        <div className="progress-bar">
                            <div className="progress-bar-indeterminate" />
                        </div>
                    </div>
                )}

                {/* Loading Message */}
                <p className="loading-message">{message}</p>

                {/* Loading Dots Animation */}
                <div className="loading-dots">
                    <span className="dot"></span>
                    <span className="dot"></span>
                    <span className="dot"></span>
                </div>
            </div>
        </div>
    );
}
