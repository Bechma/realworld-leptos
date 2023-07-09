/**
 * https://programadorwebvalencia.com/descodificar-jwt-en-javascript-o-node/
 * 
 * Decode JWT (JSON Web Token - <https://jwt.io/>) to it's subject
 * @param {string} token
 * @returns {object}
 */
export function decodeJWT(token) {
    const base64Url = token.split('.')[1];
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
    const jsonPayload = decodeURIComponent(window.atob(base64).split('').map(function (c) {
        return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2);
    }).join(''));
    return JSON.parse(jsonPayload).sub;
}

/**
 * 
 * @param {string} email 
 * @returns {boolean}
 */
export function emailRegex(email) {
    return /^[\w\-\.]+@([\w-]+\.)+\w{2,4}$/.test(email)
}