import { invokeHasApiKey } from './ipc';
import { useGraphStore } from './graphStore';

/**
 * Poll `has_api_key` and write the result into the shared key-chip
 * state (`graphStore.hasKey`) — M09.5.F honest key chip.
 *
 * Called at App mount (the M07-IRL #7 restart seed) and after every
 * settled run, EXCEPT a run that failed `SetupRequired`: there the run
 * loop's own `read_api_key` already proved no key resolves, and the
 * flip it triggered must stay authoritative — a still-true probe
 * racing in afterwards would recreate the exact lie this stage kills
 * (a failed-for-no-key run beside a green chip). Callers enforce that
 * skip; this helper only polls and writes.
 *
 * A probe error leaves the state untouched (the user can re-enter the
 * key; an app launch must not redden on a transiently locked
 * keychain) — matching the prior mount-seed behavior.
 */
export async function refreshHasKey(): Promise<void> {
  try {
    const present = await invokeHasApiKey();
    // Only write a real boolean — a malformed bridge payload must not
    // corrupt the chip state (the getCurrentTier mount-seed guard /
    // listInstalledArtifacts Array.isArray precedent).
    if (typeof present === 'boolean') {
      useGraphStore.getState().setHasKey(present);
    }
  } catch (e) {
    console.error('has_api_key error:', e);
  }
}
