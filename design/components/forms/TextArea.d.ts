import type { ChangeEventHandler, CSSProperties } from 'react';
/** List field — one process name per line. */
export interface TextAreaProps {
  value?: string;
  onChange?: ChangeEventHandler<HTMLTextAreaElement>;
  rows?: number;
  invalid?: boolean;
  style?: CSSProperties;
  [key: string]: unknown;
}
export declare function TextArea(props: TextAreaProps): JSX.Element;
