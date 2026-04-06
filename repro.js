const view = new DataView(new ArrayBuffer(4));
console.log("Before Clone: DataView length is", view.byteLength);

try {
  const cloned = structuredClone(view);
  console.log("After Clone: Cloned DataView length is", cloned.byteLength);
  console.log("✅ structuredClone with DataView is working!");
} catch (e) {
  console.log("❌ Error:", e.name, "-", e.message);
}
