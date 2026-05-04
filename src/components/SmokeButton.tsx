interface Props {
  disabled: boolean;
  onClick: () => Promise<void>;
}

export function SmokeButton({ disabled, onClick }: Props): JSX.Element {
  return (
    <button
      onClick={() => {
        void onClick();
      }}
      disabled={disabled}
      aria-label="run smoke test"
    >
      Run smoke test
    </button>
  );
}
