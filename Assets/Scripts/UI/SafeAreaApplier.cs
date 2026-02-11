using UnityEngine;
using UnityEngine.UIElements;

namespace SeekerDungeon.UI
{
    /// <summary>
    /// Reads Screen.safeArea each frame and applies matching padding to the
    /// root VisualElement of the sibling UIDocument, keeping UI content away
    /// from notches, punch-hole cameras, and rounded corners.
    /// Attach this component to the same GameObject that has the UIDocument.
    /// </summary>
    [RequireComponent(typeof(UIDocument))]
    public sealed class SafeAreaApplier : MonoBehaviour
    {
        private UIDocument _document;
        private Rect _lastSafeArea;

        private void Awake()
        {
            _document = GetComponent<UIDocument>();
        }

        private void OnEnable()
        {
            _lastSafeArea = Rect.zero; // force refresh
        }

        private void Update()
        {
            ApplySafeArea();
        }

        private void ApplySafeArea()
        {
            var root = _document?.rootVisualElement;
            if (root == null) return;

            Rect safeArea = Screen.safeArea;
            if (safeArea == _lastSafeArea) return;
            _lastSafeArea = safeArea;

            // Convert pixel safe-area to relative values against full screen.
            int screenW = Screen.width;
            int screenH = Screen.height;
            if (screenW <= 0 || screenH <= 0) return;

            float left   = safeArea.x;
            float right  = screenW - (safeArea.x + safeArea.width);
            float top    = screenH - (safeArea.y + safeArea.height); // screen Y is bottom-up
            float bottom = safeArea.y;

            root.style.paddingLeft   = new Length(left, LengthUnit.Pixel);
            root.style.paddingRight  = new Length(right, LengthUnit.Pixel);
            root.style.paddingTop    = new Length(top, LengthUnit.Pixel);
            root.style.paddingBottom = new Length(bottom, LengthUnit.Pixel);
        }
    }
}
