import type { ChangeEventHandler, CSSProperties } from 'react';
/** Single-line text field (API key, names). */
export interface TextInputProps {
  value?: string;
  onChange?: ChangeEventHandler<HTMLInputElement>;
  placeholder?: string;
  width?: number | string;
  invalid?: boolean;
  style?: CSSProperties;
  [key: string]: unknown;
}
export declare function TextInput(props: TextInputProps): JSX.Element;
