/*
 *  Permission is hereby granted, free of charge, to any person obtaining a copy
 *  of this software and associated documentation files (the "Software"), to deal
 *  in the Software without restriction, including without limitation the rights
 *  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 *  copies of the Software, and to permit persons to whom the Software is
 *  furnished to do so, subject to the following conditions:
 *
 *  The above copyright notice and this permission notice shall be included in
 *  all copies or substantial portions of the Software.
 *
 *  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 *  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 *  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 *  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 *  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 *  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 *  THE SOFTWARE.
 *
 *  Copyright(c) 2023-26 F4JDN - Jean-Michel Cohen
 *  
*/

const __BLACK__: &str = "\x1b[0;90m";
const __RED__: &str = "\x1b[0;91m";
const __GREEN__: &str = "\x1b[0;92m";
const __YELLOW__: &str = "\x1b[0;93m";
const __ORANGE__: &str = "\x1b[38;2;255;165;0m";
const __BLUE__: &str = "\x1b[0;94m";
const __MAGENTA__: &str = "\x1b[0;95m";
const __CYAN__: &str = "\x1b[0;96m";
const __WHITE__: &str = "\x1b[0;97m";

const __BOLD__: &str = "\x1b[1m";
const __CURSORON__: &str = "\x1b[?25h";
const __CURSOROFF__: &str = "\x1b[?25l";
const __ERASEEOL__: &str = "\x1b[0K";

const __CLEAR__: &str = "\x1b[2J";
const __HOME__: &str = "\x1b[2H";
const __RESET__: &str = "\x1b[0m";

const __OK__: &str = "\x1b[0;92mOK\x1b[0m";
const __WARNING__: &str = "\x1b[0;93mWarning\x1b[0m";
const __FAILED__: &str = "\x1b[0;91mFailed\x1b[0m";
