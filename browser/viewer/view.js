import * as THREE from 'three';
import {OrbitControls} from './vendor/OrbitControls.js';

export class RecordedView {
  constructor(host) {
    this.host = host;
    this.scene = new THREE.Scene(); this.scene.background = new THREE.Color(0xeff5f6);
    this.camera = new THREE.PerspectiveCamera(42, 1, .01, 1000);
    this.renderer = new THREE.WebGLRenderer({antialias: true});
    this.renderer.setPixelRatio(Math.min(devicePixelRatio, 2)); host.prepend(this.renderer.domElement);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement); this.controls.enableDamping = true;
    this.group = new THREE.Group(); this.scene.add(this.group);
    this.scene.add(new THREE.HemisphereLight(0xffffff, 0x526979, 2.8));
    const light = new THREE.DirectionalLight(0xffffff, 2); light.position.set(3, 5, 4); this.scene.add(light);
    this.observer = new ResizeObserver(() => this.resize()); this.observer.observe(host);
    this.camera.position.set(2, 1.8, 2.5);
    this.renderer.setAnimationLoop(() => { this.controls.update(); this.renderer.render(this.scene, this.camera); });
  }
  resize() {
    if (!this.host.clientWidth || !this.host.clientHeight) return;
    this.renderer.setSize(this.host.clientWidth, this.host.clientHeight);
    this.camera.aspect = this.host.clientWidth / this.host.clientHeight; this.camera.updateProjectionMatrix();
  }
  clear() {
    this.group.traverse(o => { o.geometry?.dispose(); for (const m of Array.isArray(o.material) ? o.material : [o.material]) m?.dispose(); });
    this.group.clear();
  }
  show(frame, fit = false) {
    this.clear();
    if (frame.vertices) {
      const geometry = new THREE.BufferGeometry();
      geometry.setAttribute('position', new THREE.Float32BufferAttribute(frame.vertices.flat(), 3));
      geometry.setIndex(frame.triangles.flat()); geometry.computeVertexNormals();
      this.group.add(new THREE.Mesh(geometry, new THREE.MeshStandardMaterial({color: 0x229aab, roughness: .55, side: THREE.DoubleSide})));
      const lines = new THREE.LineSegments(new THREE.EdgesGeometry(geometry), new THREE.LineBasicMaterial({color: 0x145663})); this.group.add(lines);
      if (frame.force) {
        const force = new THREE.Vector3(...frame.force.value), length = force.length();
        if (length > 0) {
          const extent = new THREE.Box3().setFromObject(this.group).getSize(new THREE.Vector3()).length();
          this.group.add(new THREE.ArrowHelper(force.normalize(), new THREE.Vector3(...frame.force.origin), extent * .6, 0xd8753f));
        }
      }
    } else {
      const speed = Math.max(1e-12, ...frame.profile.map(p => Math.abs(p[1])));
      const low = Math.min(...frame.profile.map(p => p[0])), high = Math.max(...frame.profile.map(p => p[0]));
      const span = Math.max(high - low, .5);
      for (const [y, velocity] of frame.profile) {
        const sphere = new THREE.Mesh(new THREE.SphereGeometry(span * .018, 8, 8), new THREE.MeshStandardMaterial({color: 0x1998a8}));
        sphere.position.set(0, y, 0); this.group.add(sphere);
        if (velocity !== 0) this.group.add(new THREE.ArrowHelper(new THREE.Vector3(Math.sign(velocity), 0, 0), new THREE.Vector3(0, y, 0), Math.abs(velocity) / speed * span * .8, 0x1998a8));
      }
    }
    if (fit) this.fit();
  }
  fit() {
    const bounds = new THREE.Box3().setFromObject(this.group); if (bounds.isEmpty()) return;
    const center = bounds.getCenter(new THREE.Vector3()), size = Math.max(bounds.getSize(new THREE.Vector3()).length(), .5);
    this.controls.target.copy(center); this.camera.position.copy(center).add(new THREE.Vector3(1.3, .9, 1.7).multiplyScalar(size));
    this.camera.near = size / 1000; this.camera.far = size * 100; this.camera.updateProjectionMatrix(); this.controls.update();
  }
  dispose() {
    this.observer.disconnect(); this.renderer.setAnimationLoop(null); this.controls.dispose(); this.clear(); this.renderer.dispose(); this.renderer.domElement.remove();
  }
}
