import type { ReactNode, CSSProperties } from 'react';
/** Feature grid tile. */
export interface FeatureCardProps {
  /** lucide icon name */
  icon?: string;
  title: ReactNode;
  /** icon tint — pick tones to make a grid colourful */
  tone?: 'running' | 'loading' | 'paused' | 'off' | 'error' | 'game';
  children?: ReactNode;
  style?: CSSProperties;
}
export declare function FeatureCard(props: FeatureCardProps): JSX.Element;
