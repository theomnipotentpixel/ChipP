const fs = require("fs");
const exit = require("process").exit;
let inputFile = fs.readFileSync("main.p16", {encoding: "utf-8"});

function split_str(str){
    var regx = /[^\s"]+|"([^"]*)"/gi;
    var out = [];

    do {
        //Each call to exec returns the next regex match as an array
        var match = regx.exec(str);
        if (match != null)
        {
            //Index 1 in the array is the captured group if it exists
            //Index 0 is the matched text, which we use if no captured group exists
            out.push(match[1] ? match[1] : match[0]);
        }
    } while (match != null);
    return out;
}

function op_mov(reg1, val){
    reg1 = parseInt(reg1);
    val = parseInt(val);
    return [0x01, reg1, val >> 24, val >> 16, val >> 8, val];
}

function op_store_i(reg1, addr){
    reg1 = parseInt(reg1);
    addr = parseInt(addr);
    return [0x02, reg1, addr >> 24, addr >> 16, addr >> 8, addr];
}

function op_load_i(reg1, val){
    reg1 = parseInt(reg1);
    val = parseInt(val);
    return [0x03, reg1, val >> 24, val >> 16, val >> 8, val];
}

function op_store(reg1, reg2){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    return [0x04, reg1, reg2];
}

function op_load(reg1, reg2){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    return [0x05, reg1, reg2];
}

function op_add(reg1, reg2){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    return [0x06, reg1, reg2];
}

function op_add_i(reg1, val){
    reg1 = parseInt(reg1);
    val = parseInt(val);
    return [0x07, reg1, val >> 24, val >> 16, val >> 8, val];
}

function op_sub(reg1, reg2){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    return [0x08, reg1, reg2];
}

function op_sub_i(reg1, val){
    reg1 = parseInt(reg1);
    val = parseInt(val);
    return [0x09, reg1, val >> 24, val >> 16, val >> 8, val];
}

function op_mul(reg1, reg2){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    return [0x0A, reg1, reg2];
}

function op_div(reg1, reg2){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    return [0x0B, reg1, reg2];
}

function op_jmp(label){
    label = LABELS[label];
    if(label === undefined){
        console.error("UNDEFINED LABEL!");
        return false;
    }
    return [0x0C, label >> 24, label >> 16, label >> 8, label];
}

function op_jeq(reg1, reg2, label){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    label = LABELS[label];
    if(label === undefined){
        console.error("UNDEFINED LABEL!");
        return false;
    }
    return [0x0D, reg1, reg2, label >> 24, label >> 16, label >> 8, label];
}

function op_jne(reg1, reg2, label){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    label = LABELS[label];
    if(label === undefined){
        console.error("UNDEFINED LABEL!");
        return false;
    }
    return [0x0E, reg1, reg2, label >> 24, label >> 16, label >> 8, label];
}

function op_store_str(addr1, addr2){
    addr1 = parseInt(addr1);
    addr2 = parseInt(addr2);
    return [0x0F, addr1 >> 24, addr1 >> 16, addr1 >> 8, addr1, addr2 >> 24, addr2 >> 16, addr2 >> 8, addr2];
}

function op_print_str_mem(addr){
    addr = parseInt(addr);
    return [0x10, addr >> 24, addr >> 16,addr >> 8, addr];
}

function op_print_str_rom(label){
    label = LABELS[label];
    if(label === undefined){
        console.error("UNDEFINED LABEL!");
        return false;
    }
    return [0x11, label >> 24, label >> 16, label >> 8, label];
}

function op_call(label){
    label = LABELS[label];
    if(label === undefined){
        console.error("UNDEFINED LABEL!");
        return false;
    }
    return [0x12, label >> 24, label >> 16, label >> 8, label];
}

function op_return(){
    return [0x13];
}

function op_swap_buffers(){
    return [0x14];
}

function op_draw_pixel(reg1, reg2, val){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    val = parseInt(val);
    return [0x15, reg1, reg2, val >> 24, val >> 16, val >> 8, val];
}

function op_draw_sprite(reg1, reg2, label){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    label = LABELS[label];
    if(label === undefined){
        console.error("UNDEFINED LABEL!");
        return false;
    }
    return [0x16, reg1, reg2, label >> 24, label >> 16, label >> 8, label];
}

function op_jgt(reg1, reg2, label){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    label = LABELS[label];
    if(label === undefined){
        console.error("UNDEFINED LABEL!");
        return false;
    }
    return [0x17, reg1, reg2, label >> 24, label >> 16, label >> 8, label];
}

function op_jlt(reg1, reg2, label){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    label = LABELS[label];
    if(label === undefined){
        console.error("UNDEFINED LABEL!");
        return false;
    }
    return [0x18, reg1, reg2, label >> 24, label >> 16, label >> 8, label];
}

function op_jge(reg1, reg2, label){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    label = LABELS[label];
    if(label === undefined){
        console.error("UNDEFINED LABEL!");
        return false;
    }
    return [0x19, reg1, reg2, label >> 24, label >> 16, label >> 8, label];
}

function op_jle(reg1, reg2, label){
    reg1 = parseInt(reg1);
    reg2 = parseInt(reg2);
    label = LABELS[label];
    if(label === undefined){
        console.error("UNDEFINED LABEL!");
        return false;
    }
    return [0x1A, reg1, reg2, label >> 24, label >> 16, label >> 8, label];
}

let OPS = {
    "mov": [op_mov, 5],
    "store_i": [op_store_i, 5],
    "load_i": [op_load_i, 5],
    "store": [op_store, 2],
    "load": [op_load, 2],
    "add": [op_add, 2],
    "add_i": [op_add_i, 5],
    "sub": [op_sub, 2],
    "sub_i": [op_sub_i, 5],
    "mul": [op_mul, 2],
    "div": [op_div, 2],
    "jmp": [op_jmp, 4],
    "jeq": [op_jeq, 6],
    "jne": [op_jne, 6],
    "store_str": [op_store_str, 8],
    "print_str_mem": [op_print_str_mem, 4],
    "print_str_rom": [op_print_str_rom, 4],
    "call": [op_call, 4],
    "return": [op_return, 0],
    "swap_buffers": [op_swap_buffers, 0],
    "draw_pixel": [op_draw_pixel, 6],
    "draw_sprite": [op_draw_sprite, 6],
    "jgt": [op_jgt, 6],
    "jlt": [op_jlt, 6],
    "jge": [op_jge, 6],
    "jle": [op_jle, 6],
}

let LABELS = {};

function parse_program(lines){
    let out = [];
    let line_num = 1;
    let ops = 0;
    for(line of lines){
        let params = split_str(line);
        let op = params.shift();
        if(op === "label"){
            if(params.length !== 1){
                console.error("NO LABEL SPECIFIED!\n" + "LINE: " + line_num);
                exit();
            }
            LABELS[params[0]] = ops;
        } else if(op === "string"){
            params[0] = params[0].replace(/\\n/g, "\n");
            ops += params[0].length + 1; // +1 for null termination
        } else if(op === "bytes"){
            ops += params.length;
        } else if(OPS[op] !== undefined) {
            ops += OPS[op][1] + 1; // +1 for instruction
        }
        line_num++;
    }
    console.log(LABELS);
    line_num = 1;
    for(line of lines){
        let params = split_str(line);
        let op = params.shift();
        if(op === "string"){
            params[0] = params[0].replaceAll(/\\n/g, "\n");
            out = out.concat(params[0].split("").map(c => c.charCodeAt(0)));
            out = out.concat([0]);
        } else if(op === "bytes"){
            out = out.concat(params.map(b => parseInt(b)));
        } else if(OPS[op] !== undefined){
            let ret = OPS[op][0](...params);
            if(ret === false){
                console.log(line_num);
                exit();
            }
            out = out.concat(ret);
        }
        line_num++;
    }
    return Uint8Array.from(out);
}
let t = inputFile.split("\n");
console.log(parse_program(t)[0])
fs.writeFileSync("out.p", parse_program(t));