<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  interface Props {
    isOpen: boolean;
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    variant?: 'danger' | 'warning' | 'primary';
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    isOpen,
    title,
    message,
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    variant = 'primary',
    onConfirm,
    onCancel
  }: Props = $props();

  let dialogRef: HTMLDialogElement | null = $state(null);

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      onCancel();
    } else if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
      onConfirm();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === dialogRef) {
      onCancel();
    }
  }

  $effect(() => {
    if (isOpen && dialogRef) {
      dialogRef.showModal();
      document.addEventListener('keydown', handleKeydown);
    } else if (!isOpen && dialogRef) {
      dialogRef.close();
      document.removeEventListener('keydown', handleKeydown);
    }

    return () => {
      document.removeEventListener('keydown', handleKeydown);
    };
  });

  const variantStyles = {
    danger: 'var(--danger)',
    warning: 'var(--warning)',
    primary: 'var(--accent)'
  };

  const confirmBgColors = {
    danger: 'var(--danger)',
    warning: 'var(--warning)',
    primary: 'var(--accent)'
  };

  const confirmHoverColors = {
    danger: 'var(--danger-hover)',
    warning: '#d97706',
    primary: 'var(--accent-hover)'
  };
</script>

{#if isOpen}
  <dialog
    bind:this={dialogRef}
    class="modal"
    onclick={handleBackdropClick}
    aria-labelledby="modal-title"
    aria-describedby="modal-message"
  >
    <div class="modal-content">
      <header class="modal-header">
        <h2 id="modal-title" class="modal-title" style:color={variantStyles[variant]}>
          {title}
        </h2>
      </header>

      <div class="modal-body">
        <p id="modal-message" class="modal-message">{message}</p>
      </div>

      <footer class="modal-footer">
        <button
          type="button"
          class="btn btn-cancel"
          onclick={onCancel}
        >
          {cancelText}
        </button>
        <button
          type="button"
          class="btn btn-confirm"
          style:background-color={confirmBgColors[variant]}
          style:--hover-color={confirmHoverColors[variant]}
          onclick={onConfirm}
        >
          {confirmText}
        </button>
      </footer>
    </div>
  </dialog>
{/if}

<style>
  .modal {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: 0;
    max-width: 420px;
    width: 90%;
    color: var(--text-primary);
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  }

  .modal::backdrop {
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
  }

  .modal-content {
    padding: var(--space-6);
  }

  .modal-header {
    margin-bottom: var(--space-4);
  }

  .modal-title {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0;
  }

  .modal-body {
    margin-bottom: var(--space-6);
  }

  .modal-message {
    color: var(--text-secondary);
    line-height: 1.6;
    margin: 0;
  }

  .modal-footer {
    display: flex;
    gap: var(--space-3);
    justify-content: flex-end;
  }

  .btn {
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    font-size: 0.875rem;
    font-weight: 500;
    transition: all var(--transition-fast);
    border: 1px solid transparent;
  }

  .btn-cancel {
    background: transparent;
    border-color: var(--border-color);
    color: var(--text-secondary);
  }

  .btn-cancel:hover {
    background: var(--bg-surface-hover);
    color: var(--text-primary);
  }

  .btn-confirm {
    color: white;
  }

  .btn-confirm:hover {
    background-color: var(--hover-color) !important;
  }
</style>
