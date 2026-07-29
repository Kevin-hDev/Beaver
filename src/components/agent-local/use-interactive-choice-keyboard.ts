import {
  useCallback,
  useRef,
  type Dispatch,
  type KeyboardEvent as ReactKeyboardEvent,
  type SetStateAction,
} from "react";
import type { AgentInteractiveOption } from "@/types/agent";
import { useFocusedPanel } from "./use-focused-panel";

interface InteractiveChoiceKeyboardParams {
  options: AgentInteractiveOption[];
  activeIndex: number;
  setActiveIndex: Dispatch<SetStateAction<number>>;
  choose: (option: AgentInteractiveOption) => void;
  cancel: () => void;
  submitOther: () => void;
  closeOther: () => void;
}

export function useInteractiveChoiceKeyboard({
  options,
  activeIndex,
  setActiveIndex,
  choose,
  cancel,
  submitOther,
  closeOther,
}: InteractiveChoiceKeyboardParams) {
  const optionsRef = useRef<HTMLDivElement>(null);

  const focusOption = useCallback((index: number) => {
    optionsRef.current
      ?.querySelectorAll<HTMLButtonElement>("button")
      .item(index)
      .focus();
  }, []);

  const move = useCallback((offset: number) => {
    const nextIndex = (activeIndex + offset + options.length) % options.length;
    setActiveIndex(nextIndex);
    focusOption(nextIndex);
  }, [activeIndex, focusOption, options.length, setActiveIndex]);

  const onPanelKeyDown = useCallback((event: KeyboardEvent) => {
    if (event.target !== event.currentTarget) return;
    if (event.key === "ArrowUp") {
      event.preventDefault();
      move(-1);
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      move(1);
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      cancel();
    }
  }, [cancel, move]);

  const panelRef = useFocusedPanel(onPanelKeyDown);

  const onChoiceKeyDown = useCallback((event: ReactKeyboardEvent<HTMLElement>) => {
    if (event.key === "ArrowUp") {
      event.preventDefault();
      event.stopPropagation();
      move(-1);
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      event.stopPropagation();
      move(1);
    } else if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      event.stopPropagation();
      const option = options[activeIndex];
      if (option) choose(option);
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      cancel();
    }
  }, [activeIndex, cancel, choose, move, options]);

  const onOtherKeyDown = useCallback((event: ReactKeyboardEvent<HTMLElement>) => {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      event.stopPropagation();
      submitOther();
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      focusOption(activeIndex);
      closeOther();
    }
  }, [activeIndex, closeOther, focusOption, submitOther]);

  return { panelRef, optionsRef, onChoiceKeyDown, onOtherKeyDown };
}
