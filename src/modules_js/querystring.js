function e(e, n) {
	return Object.prototype.hasOwnProperty.call(e, n);
}
var n = function (n, r, t, o) {
		(r = r || "&"), (t = t || "=");
		var a = {};
		if ("string" !== typeof n || 0 === n.length) {
			return a;
		}
		var u = /\+/g;
		n = n.split(r);
		var c = 1e3;
		o && "number" === typeof o.maxKeys && (c = o.maxKeys);
		var i = n.length;
		c > 0 && i > c && (i = c);
		for (var s = 0; s < i; ++s) {
			var p,
				f,
				d,
				y,
				m = n[s].replace(u, "%20"),
				l = m.indexOf(t);
			l >= 0 ? ((p = m.substr(0, l)), (f = m.substr(l + 1))) : ((p = m), (f = "")),
				(d = decodeURIComponent(p)),
				(y = decodeURIComponent(f)),
				e(a, d) ? (Array.isArray(a[d]) ? a[d].push(y) : (a[d] = [a[d], y])) : (a[d] = y);
		}
		return a;
	},
	r = function (e) {
		switch (typeof e) {
			case "string":
				return e;
			case "boolean":
				return e ? "true" : "false";
			case "number":
				return isFinite(e) ? e : "";
			default:
				return "";
		}
	},
	t = function (e, n, t, o) {
		return (
			(n = n || "&"),
			(t = t || "="),
			null === e && (e = void 0),
			"object" === typeof e
				? Object.keys(e)
						.map(function (o) {
							var a = encodeURIComponent(r(o)) + t;
							return Array.isArray(e[o])
								? e[o]
										.map(function (e) {
											return a + encodeURIComponent(r(e));
										})
										.join(n)
								: a + encodeURIComponent(r(e[o]));
						})
						.join(n)
				: o
				? encodeURIComponent(r(o)) + t + encodeURIComponent(r(e))
				: ""
		);
	},
	o = {};
(o.decode = o.parse = n), (o.encode = o.stringify = t);
o.decode;
o.encode;
o.parse;
o.stringify;

o.decode;
o.encode;
o.parse;
o.stringify;

var decode = o.decode;
var encode = o.encode;
var parse = o.parse;
var stringify = o.stringify;

export { decode, o as default, encode, parse, stringify };
