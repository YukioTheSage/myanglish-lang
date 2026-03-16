"use client";
import { useState, useCallback, useEffect, useRef } from "react";

const SWIPE_THRESHOLD = 60;
const SWIPE_DOMINANCE_RATIO = 1.2;
const WHEEL_NAV_TRIGGER = 90;
const WHEEL_RESET_MS = 180;
const WHEEL_LOCK_MS = 650;

function isInteractiveElement(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false;
  return target.closest("a, button, input, textarea, select, label, summary, [role='button'], [contenteditable='true']") !== null;
}

function isVerticallyScrollable(el: HTMLElement): boolean {
  const style = window.getComputedStyle(el);
  const canScrollY =
    style.overflowY === "auto" || style.overflowY === "scroll" || style.overflowY === "overlay";

  return canScrollY && el.scrollHeight > el.clientHeight + 2;
}

function canScrollVertically(el: HTMLElement, deltaY: number): boolean {
  const maxScrollTop = el.scrollHeight - el.clientHeight;
  if (maxScrollTop <= 1) return false;

  if (deltaY > 0) return el.scrollTop < maxScrollTop - 1;
  return el.scrollTop > 1;
}

function anyScrollableAncestorCanScroll(target: EventTarget | null, deltaY: number): boolean {
  let el: Element | null = target instanceof Element ? target : null;

  while (el && el !== document.body) {
    if (el instanceof HTMLElement && isVerticallyScrollable(el) && canScrollVertically(el, deltaY)) {
      return true;
    }

    el = el.parentElement;
  }

  const slideRoot = document.querySelector<HTMLElement>("[data-slide-scroll-root]");
  return slideRoot ? canScrollVertically(slideRoot, deltaY) : false;
}

function findHorizontalScrollableAncestor(target: EventTarget | null): HTMLElement | null {
  if (!(target instanceof Element)) return null;

  let el: Element | null = target;
  while (el && el !== document.body) {
    if (el instanceof HTMLElement) {
      if (el.hasAttribute("data-slide-nav-lock-x")) return el;

      const style = window.getComputedStyle(el);
      const canScrollX =
        (style.overflowX === "auto" || style.overflowX === "scroll" || style.overflowX === "overlay") &&
        el.scrollWidth > el.clientWidth + 2;

      if (canScrollX) return el;
    }

    el = el.parentElement;
  }

  return null;
}

export function useSlideNavigation(totalSlides: number) {
  const [currentSlide, setCurrentSlide] = useState(0);
  const [direction, setDirection] = useState(0);
  const isAnimating = useRef(false);
  const wheelTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);
  const wheelAccumulator = useRef(0);
  const wheelResetTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);
  const touchStartX = useRef(0);
  const touchStartY = useRef(0);
  const touchStartedOnInteractive = useRef(false);
  const touchScrollableParent = useRef<HTMLElement | null>(null);
  const touchScrollableStartLeft = useRef(0);

  const goToSlide = useCallback(
    (target: number) => {
      if (isAnimating.current || target < 0 || target >= totalSlides || target === currentSlide) return;
      isAnimating.current = true;
      setDirection(target > currentSlide ? 1 : -1);
      setCurrentSlide(target);
      setTimeout(() => {
        isAnimating.current = false;
      }, 500);
    },
    [currentSlide, totalSlides]
  );

  const next = useCallback(() => goToSlide(currentSlide + 1), [currentSlide, goToSlide]);
  const prev = useCallback(() => goToSlide(currentSlide - 1), [currentSlide, goToSlide]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowRight":
        case "ArrowDown":
        case " ":
        case "PageDown":
          e.preventDefault();
          next();
          break;
        case "ArrowLeft":
        case "ArrowUp":
        case "PageUp":
          e.preventDefault();
          prev();
          break;
        case "Home":
          e.preventDefault();
          goToSlide(0);
          break;
        case "End":
          e.preventDefault();
          goToSlide(totalSlides - 1);
          break;
        case "f":
        case "F":
          if (!document.fullscreenElement) {
            document.documentElement.requestFullscreen().catch(() => {});
          } else {
            document.exitFullscreen().catch(() => {});
          }
          break;
      }
      if (e.key >= "1" && e.key <= "9") {
        const target = parseInt(e.key) - 1;
        if (target < totalSlides) goToSlide(target);
      }
    };

    const handleWheel = (e: WheelEvent) => {
      if (e.ctrlKey) return;
      if (Math.abs(e.deltaY) <= Math.abs(e.deltaX)) return;
      if (Math.abs(e.deltaY) < 1) return;

      if (anyScrollableAncestorCanScroll(e.target, e.deltaY)) {
        wheelAccumulator.current = 0;
        return;
      }

      if (wheelAccumulator.current !== 0 && Math.sign(wheelAccumulator.current) !== Math.sign(e.deltaY)) {
        wheelAccumulator.current = 0;
      }

      wheelAccumulator.current += e.deltaY;

      if (wheelResetTimeout.current) {
        clearTimeout(wheelResetTimeout.current);
      }
      wheelResetTimeout.current = setTimeout(() => {
        wheelAccumulator.current = 0;
        wheelResetTimeout.current = null;
      }, WHEEL_RESET_MS);

      if (Math.abs(wheelAccumulator.current) < WHEEL_NAV_TRIGGER) return;
      if (wheelTimeout.current) return;

      wheelTimeout.current = setTimeout(() => {
        wheelTimeout.current = null;
      }, WHEEL_LOCK_MS);

      const directionDelta = wheelAccumulator.current;
      wheelAccumulator.current = 0;

      if (directionDelta > 0) next();
      else if (directionDelta < 0) prev();
    };

    const handleTouchStart = (e: TouchEvent) => {
      touchStartX.current = e.changedTouches[0].screenX;
      touchStartY.current = e.changedTouches[0].screenY;
      touchStartedOnInteractive.current = isInteractiveElement(e.target);
      touchScrollableParent.current = findHorizontalScrollableAncestor(e.target);
      touchScrollableStartLeft.current = touchScrollableParent.current?.scrollLeft ?? 0;
    };

    const handleTouchEnd = (e: TouchEvent) => {
      if (touchStartedOnInteractive.current) return;

      const scrollContainer = touchScrollableParent.current;
      if (scrollContainer) {
        const localScrollDelta = Math.abs(scrollContainer.scrollLeft - touchScrollableStartLeft.current);
        if (localScrollDelta > 6) return;
      }

      const deltaX = touchStartX.current - e.changedTouches[0].screenX;
      const deltaY = touchStartY.current - e.changedTouches[0].screenY;
      const absX = Math.abs(deltaX);
      const absY = Math.abs(deltaY);

      if (absX < SWIPE_THRESHOLD) return;
      if (absX < absY * SWIPE_DOMINANCE_RATIO) return;

      if (deltaX > 0) next();
      else prev();
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("wheel", handleWheel, { passive: true });
    window.addEventListener("touchstart", handleTouchStart, { passive: true });
    window.addEventListener("touchend", handleTouchEnd, { passive: true });

    return () => {
      if (wheelTimeout.current) {
        clearTimeout(wheelTimeout.current);
      }
      if (wheelResetTimeout.current) {
        clearTimeout(wheelResetTimeout.current);
      }

      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("wheel", handleWheel);
      window.removeEventListener("touchstart", handleTouchStart);
      window.removeEventListener("touchend", handleTouchEnd);
    };
  }, [next, prev, goToSlide, totalSlides]);

  const progress = ((currentSlide + 1) / totalSlides) * 100;

  return { currentSlide, direction, progress, next, prev, goToSlide, totalSlides };
}
