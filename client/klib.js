/**
 * @typedef {{class?: string, style?: object, id?: string, textContent?: string}} KProps
 */


/** 
 * Creates an element
 * @param {string} tag
 * @param {KProps} props?
 * @returns {HTMLElement}
 */
export function create(tag, props) {
    let elem = document.createElement(tag);
    if (props && props.class) {
        elem.className = props.class;
    }
    if (props && props.style) {
        for (let prop of props.style) {
            elem.style[prop] = props.style[prop];
        }
    }
    if (props && props.id) {
        elem.id = props.id;
    }
    if (props && props.textContent) {
        elem.textContent = props.textContent;
    }
    return elem;
};

/**
 * Alias for document.getElementById
 * 
 * @param {id} string
 * @returns {HTMLElement | null}
 */
export function byID(id) {
    return document.getElementById(id);
}

export function xhr(method, url) {
    return new Promise(function(resolve, reject) {
        let x = new XMLHttpRequest();
        x.onload = function() {
            resolve(x.responseText);
        };
        x.onerror = function() {
            reject(x.responseText);
        }
        x.open(method, url);
        x.send();
    });
};