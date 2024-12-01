const { Jimp } = require("jimp");


(async () => {
    try {
        const text = await Jimp.read("test.png");
        console.log(text.bitmap.data);
    } catch (e) {
        // Deal with the fact the chain failed
    }
    // `text` is not available here
})();