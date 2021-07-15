/// Replaces tags of the form <link href="footer.html" /> with the actual content.
window.addEventListener('load', (_) => {
    const allElements = document.getElementsByTagName("link");
    for (let i = 0; i < allElements.length; i++) {
        const element = allElements[i];
        const url = element.getAttribute("include");
        if (url) {
            const xhttp = new XMLHttpRequest();
            xhttp.onreadystatechange = () => {
                if (this.readyState == 4 && this.status == 200) {
                    element.innerHTML = this.responseText;
                    element.removeAttribute("include");
                }
            }
            xhttp.open("GET", url, true);
            xhttp.send();
            return;
        }
    }
});

/// Localizes the document's content according to the current locale.
///
/// For example, the following snippet sets the content of the elemnt with id `hello` to either
/// "Hello, world!" or "Hallo, Welt!", depending on the locale.
///
/// ```
/// localize({
///   "en": { "hello": "Hello, world!" },
///   "de": { "hello": "Hallo, Welt!" }
/// });
/// ```
function localize(translations) {
    function actuallyLocalize() {
        var lang = translations[navigator.language.substr(0, 2)] ?? translations["en"];
        for (let key in lang) {
            let element = document.getElementById(key);
            if (element) {
                element.innerHTML = lang[key];
            }
        }
    }
    window.addEventListener('load', (event) => {
        actuallyLocalize();
    });
    window.addEventListener('languagechange', (event) => {
        actuallyLocalize();
    });
}
