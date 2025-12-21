// apps/designer/src/hooks/useFileHandlers.ts
import { useRef, useState, useCallback } from "react";
import {
    useArchitectureStore,
    useToastStore,
    useUIStore,
} from "../stores";
import { firebaseShareService } from "../utils/firebaseShareService";
import { convertModelToDsl } from "../utils/modelToDsl";
import { convertDslToLikeC4 } from "../wasm";
import type { SrujaModelDump } from "@sruja/shared";
import { handleError, getUserFriendlyMessage, safeAsync, ErrorType } from "../utils/errorHandling";

/**
 * Hook for handling file operations: import, export, share, and create new projects.
 */
export function useFileHandlers(canvasRef: React.RefObject<any>) { // simplified ref type
    const likec4Model = useArchitectureStore((s) => s.likec4Model);
    const storeDslSource = useArchitectureStore((s) => s.dslSource);
    const loadFromDSL = useArchitectureStore((s) => s.loadFromDSL);
    const setActiveTab = useUIStore((s) => s.setActiveTab);
    const showToast = useToastStore((s) => s.showToast);

    const [isImporting, setIsImporting] = useState(false);
    const fileInputRef = useRef<HTMLInputElement>(null);

    const handleShare = useCallback(async () => {
        const dsl = storeDslSource || (likec4Model ? await convertModelToDsl(likec4Model) : "");
        if (!dsl) return;

        const { error: shareError } = await safeAsync(
            async () => {
                const { projectId, keyBase64 } = firebaseShareService.parseUrl(window.location.href);
                const currentProjectId = firebaseShareService.getCurrentProjectId();

                let shareUrl: string;

                if (projectId && keyBase64 && projectId === currentProjectId) {
                    // Try to save to existing project
                    const { error: saveError } = await safeAsync(
                        () => firebaseShareService.saveProject(dsl),
                        "Failed to save project",
                        ErrorType.NETWORK
                    );

                    if (saveError) {
                        // Fallback: create new project
                        const { error: createError, data: newProject } = await safeAsync(
                            () => firebaseShareService.createNewProject(),
                            "Failed to create new project",
                            ErrorType.NETWORK
                        );

                        if (createError || !newProject) throw createError || new Error("Failed to create project");

                        const { error: saveError2 } = await safeAsync(
                            () => firebaseShareService.saveProject(dsl),
                            "Failed to save project",
                            ErrorType.NETWORK
                        );

                        if (saveError2) throw saveError2;

                        shareUrl = await firebaseShareService.buildShareUrl(
                            newProject.projectId,
                            newProject.keyBase64
                        );
                        window.history.replaceState({}, "", shareUrl);
                    } else {
                        shareUrl = window.location.href;
                    }
                } else {
                    // Create new project
                    const { error: createError, data: newProject } = await safeAsync(
                        () => firebaseShareService.createNewProject(),
                        "Failed to create new project",
                        ErrorType.NETWORK
                    );

                    if (createError || !newProject) throw createError || new Error("Failed to create project");

                    const { error: saveError } = await safeAsync(
                        () => firebaseShareService.saveProject(dsl),
                        "Failed to save project",
                        ErrorType.NETWORK
                    );

                    if (saveError) throw saveError;

                    shareUrl = await firebaseShareService.buildShareUrl(
                        newProject.projectId,
                        newProject.keyBase64
                    );
                    window.history.replaceState({}, "", shareUrl);
                }

                // Copy to clipboard
                await navigator.clipboard.writeText(shareUrl);
                return shareUrl;
            },
            "Failed to generate share URL",
            ErrorType.NETWORK
        );

        if (shareError) {
            const error = handleError(shareError, "handleShare");
            showToast(getUserFriendlyMessage(error), "error");
            return;
        }

        showToast(
            "Link copied to clipboard!\n\n⚠️ Anyone with this link can view and edit.\n⚠️ We cannot recover it if you lose it.\n\n💡 Tip: Export your DSL to keep a personal backup.",
            "success",
            8000
        );
    }, [storeDslSource, likec4Model, showToast]);

    const handleExport = useCallback(async () => {
        if (!likec4Model) return;
        const dsl = storeDslSource || await convertModelToDsl(likec4Model);
        const blob = new Blob([dsl], { type: "text/plain" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        // Access name from metadata safe access
        const name = likec4Model._metadata?.name || "architecture";
        a.download = `${name}.sruja`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }, [likec4Model, storeDslSource]);

    const handleExportPNG = useCallback(async () => {
        if (!canvasRef.current || !likec4Model) return;

        const { error } = await safeAsync(
            () => canvasRef.current!.exportAsPNG(),
            "Failed to export PNG",
            ErrorType.UNKNOWN
        );

        if (error) {
            const handledError = handleError(error, "handleExportPNG");
            showToast(getUserFriendlyMessage(handledError), "error");
        }
    }, [likec4Model, canvasRef, showToast]);

    const handleExportSVG = useCallback(async () => {
        if (!canvasRef.current || !likec4Model) return;

        const { error } = await safeAsync(
            () => canvasRef.current!.exportAsSVG(),
            "Failed to export SVG",
            ErrorType.UNKNOWN
        );

        if (error) {
            const handledError = handleError(error, "handleExportSVG");
            showToast(getUserFriendlyMessage(handledError), "error");
        }
    }, [likec4Model, canvasRef, showToast]);

    const handleImport = useCallback(() => {
        fileInputRef.current?.click();
    }, []);

    const reloadFromDsl = useCallback(async () => {
        localStorage.removeItem("architecture-visualizer-feature-flags");
        const { dslSource, currentExampleFile } = useArchitectureStore.getState();
        if (dslSource) {
            const { error, data: json } = await safeAsync(
                () => convertDslToLikeC4(dslSource),
                "Failed to convert DSL",
                ErrorType.VALIDATION
            );

            if (error) {
                handleError(error, "reloadFromDsl");
                return;
            }

            if (json) {
                await loadFromDSL(json as SrujaModelDump, dslSource, currentExampleFile ?? undefined);
                setActiveTab("diagram");
            }
        }
    }, [loadFromDSL, setActiveTab]);

    const handleFileChange = useCallback(
        async (event: React.ChangeEvent<HTMLInputElement>) => {
            const file = event.target.files?.[0];
            if (!file) return;

            setIsImporting(true);

            const { error } = await safeAsync(
                async () => {
                    const text = await file.text();
                    const { error: convertError, data: json } = await safeAsync(
                        () => convertDslToLikeC4(text),
                        "Failed to parse file",
                        ErrorType.VALIDATION
                    );

                    if (convertError || !json) {
                        throw convertError || new Error("Invalid file format");
                    }

                    const { error: createError, data: newProject } = await safeAsync(
                        () => firebaseShareService.createNewProject(),
                        "Failed to create project",
                        ErrorType.NETWORK
                    );

                    if (createError || !newProject) {
                        throw createError || new Error("Failed to create project");
                    }

                    const { error: saveError } = await safeAsync(
                        () => firebaseShareService.saveProject(text),
                        "Failed to save project",
                        ErrorType.NETWORK
                    );

                    if (saveError) throw saveError;

                    const shareUrl = await firebaseShareService.buildShareUrl(
                        newProject.projectId,
                        newProject.keyBase64
                    );
                    window.history.replaceState({}, "", shareUrl);

                    loadFromDSL(json as SrujaModelDump, text, file.name);
                    return true;
                },
                "Failed to load file",
                ErrorType.UNKNOWN
            );

            if (error) {
                const handledError = handleError(error, "handleFileChange");
                showToast(getUserFriendlyMessage(handledError), "error");
            }

            setIsImporting(false);
            if (fileInputRef.current) {
                fileInputRef.current.value = "";
            }
        },
        [loadFromDSL, showToast]
    );

    const handleCreateNew = useCallback(async () => {
        const emptyDsl = `specification {
  element person
  element system
}

model {
}`;

        const { error, data: json } = await safeAsync(
            () => convertDslToLikeC4(emptyDsl),
            "Failed to create project template",
            ErrorType.VALIDATION
        );

        if (error || !json) {
            const handledError = handleError(error || new Error("Invalid template"), "handleCreateNew");
            showToast(getUserFriendlyMessage(handledError), "error");
            return;
        }

        const { error: createError, data: newProject } = await safeAsync(
            () => firebaseShareService.createNewProject(),
            "Failed to create project",
            ErrorType.NETWORK
        );

        if (createError || !newProject) {
            const handledError = handleError(createError || new Error("Failed to create project"), "handleCreateNew");
            showToast(getUserFriendlyMessage(handledError), "error");
            return;
        }

        const shareUrl = await firebaseShareService.buildShareUrl(
            newProject.projectId,
            newProject.keyBase64
        );
        window.history.replaceState({}, "", shareUrl);

        loadFromDSL(json as SrujaModelDump, emptyDsl, undefined);
        setActiveTab("diagram");
    }, [loadFromDSL, setActiveTab, showToast]);

    const handleCreateLocal = useCallback(async () => {
        const emptyDsl = `specification {
  element person
  element system
}

model {
  user = person "User"
  web = system "WebApp"
  user -> web "uses"
}`;
        // Create full JSON structure from DSL dynamically instead of hardcoding legacy layout
        // Assuming convertDslToLikeC4 is available locally or we just use DSL
        const { data: json } = await safeAsync(() => convertDslToLikeC4(emptyDsl), "Create Local", ErrorType.VALIDATION);

        if (json) {
            loadFromDSL(json as SrujaModelDump, emptyDsl, "new");
            setActiveTab("diagram");
        } else {
            showToast("Failed to create local project", "error");
        }
    }, [loadFromDSL, setActiveTab, showToast]);

    return {
        handleShare,
        handleExport,
        handleExportPNG,
        handleExportSVG,
        handleImport,
        handleFileChange,
        handleCreateNew,
        handleCreateLocal,
        reloadFromDsl,
        isImporting,
        fileInputRef,
    };
}
