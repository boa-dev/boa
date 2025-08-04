// https://developer.mozilla.org/en-US/docs/Web/API/Window/structuredClone#cloning_an_object

const mushrooms1 = {
    amanita: ["muscaria", "virosa"],
};

const mushrooms2 = structuredClone(mushrooms1);

assertNEq(mushrooms1, mushrooms2);
assertArrayEqual(mushrooms1.amanita, mushrooms2.amanita);

mushrooms2.amanita.push("pantherina");
mushrooms1.amanita.pop();

assertArrayEqual(mushrooms2.amanita, ["muscaria", "virosa", "pantherina"]);
assertArrayEqual(mushrooms1.amanita, ["muscaria"]);
