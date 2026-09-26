import type { ReactNode, CSSProperties } from 'react';
/** Marketing window chrome. */
export interface WindowFrameProps {
  caption?: ReactNode;
  children?: ReactNode;
  style?: CSSProperties;
}
export declare function WindowFrame(props: WindowFrameProps): JSX.Element;
