"use client";
import { motion } from "framer-motion";

interface SlideNavProps {
  currentSlide: number;
  totalSlides: number;
  progress: number;
  onPrev: () => void;
  onNext: () => void;
}

export default function SlideNav({ currentSlide, totalSlides, progress, onPrev, onNext }: SlideNavProps) {
  const isFirst = currentSlide === 0;
  const isLast = currentSlide === totalSlides - 1;

  return (
    <>
      {/* Progress Bar */}
      <div className="fixed top-0 left-0 w-full h-[3px] bg-white/5 z-50">
        <motion.div
          className="h-full rounded-r-sm"
          style={{ background: "linear-gradient(135deg, #60a5fa, #a78bfa, #f472b6)" }}
          animate={{ width: `${progress}%` }}
          transition={{ duration: 0.4, ease: "easeOut" }}
        />
      </div>

      {/* Slide Counter */}
      <div className="fixed top-4 right-4 sm:top-auto sm:bottom-6 sm:right-8 text-[0.76rem] sm:text-[0.8rem] font-medium text-text-muted z-50 font-mono tracking-wider">
        {currentSlide + 1} / {totalSlides}
      </div>

      {/* Desktop Nav Arrows */}
      <button
        onClick={onPrev}
        className="hidden sm:flex fixed left-4 top-1/2 -translate-y-1/2 w-12 h-12 border border-white/10 bg-[#111827]/80 backdrop-blur-xl text-text-secondary text-2xl rounded-full items-center justify-center z-50 transition-all hover:bg-accent-blue/15 hover:border-accent-blue/30 hover:text-accent-blue hover:shadow-[0_0_30px_rgba(96,165,250,0.15)] cursor-pointer"
        style={{ opacity: isFirst ? 0.3 : 1, pointerEvents: isFirst ? "none" : "auto" }}
        aria-label="Previous slide"
      >
        &#8249;
      </button>
      <button
        onClick={onNext}
        className="hidden sm:flex fixed right-4 top-1/2 -translate-y-1/2 w-12 h-12 border border-white/10 bg-[#111827]/80 backdrop-blur-xl text-text-secondary text-2xl rounded-full items-center justify-center z-50 transition-all hover:bg-accent-blue/15 hover:border-accent-blue/30 hover:text-accent-blue hover:shadow-[0_0_30px_rgba(96,165,250,0.15)] cursor-pointer"
        style={{ opacity: isLast ? 0.3 : 1, pointerEvents: isLast ? "none" : "auto" }}
        aria-label="Next slide"
      >
        &#8250;
      </button>

      {/* Mobile Nav */}
      <div className="fixed sm:hidden bottom-4 left-1/2 -translate-x-1/2 z-50 flex items-center gap-2 rounded-full border border-white/10 bg-[#111827]/85 backdrop-blur-xl px-2 py-1.5">
        <button
          onClick={onPrev}
          className="w-10 h-10 border border-white/10 bg-bg-secondary text-text-secondary text-xl rounded-full flex items-center justify-center disabled:opacity-30"
          disabled={isFirst}
          aria-label="Previous slide"
        >
          &#8249;
        </button>
        <button
          onClick={onNext}
          className="w-10 h-10 border border-white/10 bg-bg-secondary text-text-secondary text-xl rounded-full flex items-center justify-center disabled:opacity-30"
          disabled={isLast}
          aria-label="Next slide"
        >
          &#8250;
        </button>
      </div>
    </>
  );
}
