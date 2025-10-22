export type Item = {
  site: string;
  title: string;
  price: string;
  currency: string;
  price_usd: string;
  link: string;
  image: string;
  match_confidence: number;
};

export interface CompareResponse {
  best_deal: Item;
  all_prices: Item[];
}