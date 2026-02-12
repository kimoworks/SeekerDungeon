using System;
using System.Collections.Generic;
using SeekerDungeon.Solana;
using UnityEngine;

namespace SeekerDungeon.Dungeon
{
    [Serializable]
    public sealed class DoorStateVisualEntry
    {
        [SerializeField] private RoomWallState wallState;
        [SerializeField] private GameObject visualRoot;

        public RoomWallState WallState => wallState;
        public GameObject VisualRoot => visualRoot;
    }

    public sealed class DoorVisualController : MonoBehaviour
    {
        [SerializeField] private List<DoorStateVisualEntry> stateVisuals = new();
        [SerializeField] private GameObject fallbackVisualRoot;

        private readonly Dictionary<RoomWallState, GameObject> _visualByState = new();

        /// <summary>
        /// The VisualInteractable on the currently active state visual, or null.
        /// Updated every time ApplyDoorState swaps the active child.
        /// </summary>
        public VisualInteractable ActiveVisualInteractable { get; private set; }

        private void Awake()
        {
            RebuildStateIndex();
        }

        public void ApplyDoorState(DoorJobView door)
        {
            if (door == null)
            {
                return;
            }

            var targetVisual = ResolveVisual(door.WallState);

            foreach (var stateVisual in _visualByState.Values)
            {
                if (stateVisual == null)
                {
                    continue;
                }

                stateVisual.SetActive(stateVisual == targetVisual);
            }

            if (fallbackVisualRoot != null)
            {
                var useFallback = targetVisual == null;
                fallbackVisualRoot.SetActive(useFallback);
            }

            // Resolve the VisualInteractable on whichever state visual is now active.
            ActiveVisualInteractable = targetVisual != null
                ? targetVisual.GetComponentInChildren<VisualInteractable>(true)
                : null;
        }

        private void RebuildStateIndex()
        {
            _visualByState.Clear();

            foreach (var stateVisual in stateVisuals)
            {
                if (stateVisual == null || stateVisual.VisualRoot == null)
                {
                    continue;
                }

                _visualByState[stateVisual.WallState] = stateVisual.VisualRoot;
            }
        }

        private GameObject ResolveVisual(RoomWallState wallState)
        {
            if (_visualByState.TryGetValue(wallState, out var visual))
            {
                return visual;
            }

            return null;
        }
    }
}
