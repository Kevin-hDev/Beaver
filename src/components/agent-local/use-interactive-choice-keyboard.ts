import {
  useCallback,
  useRef,
  type Dispatch,
  type KeyboardEvent,
  type SetStateAction,
} from "react";
import type { AgentInteractiveOption } from "@/types/agent";

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

  const onChoiceKeyDown = useCallback((event: KeyboardEvent<HTMLElement>) => {
    if (event.key === "ArrowUp") {
      event.preventDefault();
      event.stopPropagation();
      setActiveIndex((value) => (value - 1 + options.length) % options.length);
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      event.stopPropagation();
      setActiveIndex((value) => (value + 1) % options.length);
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
  }, [activeIndex, cancel, choose, options, setActiveIndex]);

  const onOtherKeyDown = useCallback((event: KeyboardEvent<HTMLElement>) => {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      event.stopPropagation();
      submitOther();
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      optionsRef.current?.querySelector<HTMLButtonElement>("button")?.focus();
      closeOther();
    }
  }, [closeOther, submitOther]);

  return { optionsRef, onChoiceKeyDown, onOtherKeyDown };
}
