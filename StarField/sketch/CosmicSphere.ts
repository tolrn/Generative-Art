
class CosmicSphere {
    size: number;
    pos: p5.Vector;
    col: Vec4;
    iterations: number;
    constructor(size: number, pos: p5.Vector, col: Vec4, iterations: number) {
        this.size = size;
        this.pos = pos;
        this.col = col;
        this.iterations = iterations;
    }

    drawSphere() {
        blendMode("source-over")
        fill(color(0, 0, 0, 255));
        circle(this.pos.x, this.pos.y, this.size * 2);
        blendMode(ADD);
        noFill();
        for (var i = 0; i < this.iterations; i++) {
            var p1 = randomPointOnCircleEdge(this.size, this.pos.x * 2, this.pos.y * 2);
            var p2 = randomPointOnCircleEdge(this.size, this.pos.x * 2, this.pos.y * 2);
            var lenLine = p1.dist(p2);
            var aLineLen = map(lenLine, 0, this.size * 2, 1, 0.3, true);
            var midPoint = p1.add(p2).div(2);
            var dLight = midPoint.dist(lightSource);
            var aLight = map(dLight, 600, 1200, 0, 1.0);
            var alpha_1 = aLight * aLineLen;
            var baseColor =
                color(
                    this.col.a,
                    this.col.b - alpha_1 * this.col.b,
                    this.col.c,
                    alpha_1 * this.col.d);
            stroke(baseColor);
            strokeWeight(1.0);
            line(p1.x, p1.y, p2.x, p2.y);
        }
    }
}