import type { ReactNode, CSSProperties } from 'react';
/** FAQ disclosure. */
export interface FaqItemProps {
  question: ReactNode;
  defaultOpen?: boolean;
  children?: ReactNode;
  style?: CSSProperties;
}
export declare function FaqItem(props: FaqItemProps): JSX.Element;
