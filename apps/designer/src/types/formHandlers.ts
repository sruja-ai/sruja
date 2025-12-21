// apps/designer/src/types/formHandlers.ts
// Type definitions for form event handlers

import type { FormEvent, ChangeEvent } from "react";

/**
 * Typed form submit event
 */
export interface FormSubmitEvent extends FormEvent<HTMLFormElement> {
  preventDefault: () => void;
  currentTarget: HTMLFormElement;
}

/**
 * Typed form submit handler
 */
export type FormSubmitHandler = (e: FormSubmitEvent) => void | Promise<void>;

/**
 * Typed input change event
 */
export interface InputChangeEvent extends ChangeEvent<HTMLInputElement> {
  target: HTMLInputElement;
}

/**
 * Typed textarea change event
 */
export interface TextareaChangeEvent extends ChangeEvent<HTMLTextAreaElement> {
  target: HTMLTextAreaElement;
}

/**
 * Typed select change event
 */
export interface SelectChangeEvent extends ChangeEvent<HTMLSelectElement> {
  target: HTMLSelectElement;
}

/**
 * Generic change event handler
 */
export type ChangeEventHandler<T extends HTMLElement = HTMLElement> = (
  e: ChangeEvent<T>
) => void;

/**
 * Form data extractor function type
 */
export type FormDataExtractor<T> = (form: HTMLFormElement) => T;

/**
 * Form validator function type
 */
export type FormValidator<T> = (data: T) => {
  isValid: boolean;
  errors: Record<string, string>;
};
