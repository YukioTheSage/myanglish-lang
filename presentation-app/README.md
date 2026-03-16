# M-Lang Presentation App

Interactive slide deck for the M-Lang project, built with Next.js + React + Framer Motion.

## Stack

- Next.js `16.1.6`
- React `19.2.3`
- TypeScript
- Tailwind CSS v4
- Framer Motion

## Run

```bash
npm install
npm run dev
```

Open `http://localhost:3000`.

Other scripts:

```bash
npm run build
npm run start
npm run lint
```

## Navigation

Supported controls (`src/hooks/useSlideNavigation.ts`):

- Next: `ArrowRight`, `ArrowDown`, `Space`, `PageDown`, wheel down, swipe left
- Previous: `ArrowLeft`, `ArrowUp`, `PageUp`, wheel up, swipe right
- First slide: `Home`
- Last slide: `End`
- Toggle fullscreen: `F`
- Jump to slide: keys `1`..`9` (if slide exists)

## Project Structure

```text
src/app/page.tsx                     # app entry
src/components/Presentation.tsx      # slide container + transitions
src/hooks/useSlideNavigation.ts      # keyboard/wheel/touch navigation state
src/components/slides/*              # individual slide content
src/components/ui/*                  # shared presentation UI
```

## Notes

- The deck currently includes 27 slides (see `slides` array in `Presentation.tsx`).
- Presenter notes live in `SPEAKER_NOTES.md`.
- Transitions use `AnimatePresence` + horizontal motion variants.
- This app is separate from the compiler CLI; build/run compiler from the `mlang/` root.
