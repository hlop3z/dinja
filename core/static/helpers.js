/**
 * TypeScript/JavaScript runtime helpers for Dinja
 * These helpers support legacy TypeScript decorators (experimentalDecorators)
 * Source: tslib (https://github.com/microsoft/tslib)
 */

// Decorator helper for legacy TypeScript decorators
// Applies decorators to classes and class members
if (typeof globalThis._decorate === 'undefined') {
    globalThis._decorate = function(decorators, target, key, desc) {
        var c = arguments.length;
        var r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc;
        var d;
        if (typeof Reflect === "object" && typeof Reflect.decorate === "function") {
            r = Reflect.decorate(decorators, target, key, desc);
        } else {
            for (var i = decorators.length - 1; i >= 0; i--) {
                if (d = decorators[i]) {
                    r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
                }
            }
        }
        return c > 3 && r && Object.defineProperty(target, key, r), r;
    };
}

// Metadata helper for TypeScript decorator metadata
// Used by frameworks that rely on reflect-metadata
if (typeof globalThis._decorateMetadata === 'undefined') {
    globalThis._decorateMetadata = function(metadataKey, metadataValue) {
        if (typeof Reflect === "object" && typeof Reflect.metadata === "function") {
            return Reflect.metadata(metadataKey, metadataValue);
        }
        // Return a no-op decorator if Reflect.metadata is not available
        return function() {};
    };
}

// Object spread helper for ES5 compatibility (used by Oxc transformer)
if (typeof globalThis._objectSpread === 'undefined') {
    globalThis._objectSpread = function(target) {
        for (var i = 1; i < arguments.length; i++) {
            var source = arguments[i];
            if (source != null) {
                for (var key in source) {
                    if (Object.prototype.hasOwnProperty.call(source, key)) {
                        target[key] = source[key];
                    }
                }
            }
        }
        return target;
    };
}
