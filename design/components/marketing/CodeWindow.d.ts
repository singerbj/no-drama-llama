import type { ReactNode, CSSProperties } from 'react';
/** Code sample panel. */
export interface CodeWindowProps {
  filename?: string;
  children?: ReactNode;
  style?: CSSProperties;
}
export declare function CodeWindow(props: CodeWindowProps): JSX.Element;
