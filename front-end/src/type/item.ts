export type Item = {
  id: number;
  platform: string;
  productName: string;
  price: number;
  oldPrice?: number;
  image?: string;
  timeAgo?: string;
  shipping?: string;
  delivery?: string;
  trackedSince?: string;
  status?: "Open" | "Opening" | "Tracked";
  actionLabel?: string;
  color?: string;
  link?: string;
};
