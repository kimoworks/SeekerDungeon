using DG.Tweening;
using SeekerDungeon.Solana;
using UnityEngine;

namespace SeekerDungeon.Dungeon
{
    public sealed class DungeonChestVisualController : MonoBehaviour
    {
        [SerializeField] private bool logDebugMessages;
        [SerializeField] private GameObject closedVisual;
        [SerializeField] private GameObject openVisual;

        private bool _isOpen;

        public void Apply(RoomView room)
        {
            if (room == null)
            {
                return;
            }

            if (logDebugMessages)
            {
                Debug.Log($"[DungeonChestVisual] LootedCount={room.LootedCount} HasLocalPlayerLooted={room.HasLocalPlayerLooted}");
            }

            SetOpen(room.HasLocalPlayerLooted, animate: false);
        }

        /// <summary>
        /// Immediately flip to the open state with a punch animation.
        /// Called after a successful loot transaction.
        /// </summary>
        public void PlayOpenAnimation()
        {
            if (_isOpen)
            {
                return;
            }

            SetOpen(true, animate: true);
        }

        private void SetOpen(bool open, bool animate)
        {
            _isOpen = open;

            if (closedVisual != null)
            {
                closedVisual.SetActive(!open);
            }

            if (openVisual != null)
            {
                openVisual.SetActive(open);
            }

            if (animate && open && openVisual != null)
            {
                var target = openVisual.transform;
                target.localScale = Vector3.zero;
                target.DOScale(Vector3.one, 0.35f).SetEase(Ease.OutBack);
            }
        }
    }
}
