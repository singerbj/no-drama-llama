import type { ReactNode, CSSProperties } from 'react';
/** Numbered step card. */
export interface StepCardProps {
  n: number | string;
  title: ReactNode;
  children?: ReactNode;
  style?: CSSProperties;
}
export declare function StepCard(props: StepCardProps): JSX.Element;
