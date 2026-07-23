import { useEffect } from "react";

const HOME_TITLE = "Gomoku2D — An old favorite, built properly.";

export function useDocumentTitle(page?: string): void {
  useEffect(() => {
    document.title = page ? `${page} | Gomoku2D` : HOME_TITLE;
  }, [page]);
}
