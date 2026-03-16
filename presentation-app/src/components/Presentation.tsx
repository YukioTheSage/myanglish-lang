"use client";
import { AnimatePresence, motion } from "framer-motion";
import { useSlideNavigation } from "@/hooks/useSlideNavigation";
import SlideNav from "./ui/SlideNav";
import TitleSlide from "./slides/TitleSlide";
import GroupIntroSlide from "./slides/GroupIntroSlide";
import ReferenceParadigmSlide from "./slides/ReferenceParadigmSlide";
import ReferenceContextSlide from "./slides/ReferenceContextSlide";
import ReferenceDesignSlide from "./slides/ReferenceDesignSlide";
import ReferenceBNFSlide from "./slides/ReferenceBNFSlide";
import TargetOutlineSlide from "./slides/TargetOutlineSlide";
import TargetBNFSlide from "./slides/TargetBNFSlide";
import SourcesSlide from "./slides/SourcesSlide";
import MotivationSlide from "./slides/MotivationSlide";
import GoalsSlide from "./slides/GoalsSlide";
import KeywordSlide from "./slides/KeywordSlide";
import TypeSystemSlide from "./slides/TypeSystemSlide";
import CodeExamplesSlide from "./slides/CodeExamplesSlide";
import ArchitectureSlide from "./slides/ArchitectureSlide";
import LexerSlide from "./slides/LexerSlide";
import ParserSlide from "./slides/ParserSlide";
import TypeCheckerSlide from "./slides/TypeCheckerSlide";
import CodeGenSlide from "./slides/CodeGenSlide";
import ToolingSlide from "./slides/ToolingSlide";
import TestingSlide from "./slides/TestingSlide";
import ChallengesSlide from "./slides/ChallengesSlide";
import RoadmapSlide from "./slides/RoadmapSlide";
import VisionSlide from "./slides/VisionSlide";
import ComparisonSlide from "./slides/ComparisonSlide";
import ConclusionSlide from "./slides/ConclusionSlide";
import QASlide from "./slides/QASlide";

const slides = [
  TitleSlide,
  GroupIntroSlide,
  ReferenceParadigmSlide,
  ReferenceContextSlide,
  ReferenceDesignSlide,
  ReferenceBNFSlide,
  TargetOutlineSlide,
  TargetBNFSlide,
  SourcesSlide,
  MotivationSlide,
  GoalsSlide,
  KeywordSlide,
  TypeSystemSlide,
  CodeExamplesSlide,
  ArchitectureSlide,
  LexerSlide,
  ParserSlide,
  TypeCheckerSlide,
  CodeGenSlide,
  ToolingSlide,
  TestingSlide,
  ChallengesSlide,
  RoadmapSlide,
  VisionSlide,
  ComparisonSlide,
  ConclusionSlide,
  QASlide,
];

const slideVariants = {
  enter: (direction: number) => ({
    x: direction > 0 ? 300 : -300,
    opacity: 0,
  }),
  center: {
    x: 0,
    opacity: 1,
  },
  exit: (direction: number) => ({
    x: direction > 0 ? -300 : 300,
    opacity: 0,
  }),
};

export default function Presentation() {
  const nav = useSlideNavigation(slides.length);
  const CurrentSlide = slides[nav.currentSlide];

  return (
    <div className="w-full h-[100dvh] relative overflow-hidden bg-bg-primary">
      <SlideNav
        currentSlide={nav.currentSlide}
        totalSlides={nav.totalSlides}
        progress={nav.progress}
        onPrev={nav.prev}
        onNext={nav.next}
      />

      <AnimatePresence mode="wait" custom={nav.direction}>
        <motion.div
          key={nav.currentSlide}
          data-slide-scroll-root
          custom={nav.direction}
          variants={slideVariants}
          initial="enter"
          animate="center"
          exit="exit"
          transition={{ duration: 0.4, ease: [0.4, 0, 0.2, 1] }}
          className="absolute inset-0 flex items-start justify-center overflow-y-auto overflow-x-hidden"
        >
          <div className="w-full max-w-[1200px] mx-auto px-4 sm:px-8 md:px-12 lg:px-16 xl:px-20 pt-6 sm:pt-8 lg:pt-10 pb-24 sm:pb-10">
            <CurrentSlide />
          </div>
        </motion.div>
      </AnimatePresence>
    </div>
  );
}
