import { ComponentFixture, TestBed } from '@angular/core/testing';

import { Sorter } from './sorter';

describe('Sorter', () => {
  let component: Sorter;
  let fixture: ComponentFixture<Sorter>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Sorter],
    }).compileComponents();

    fixture = TestBed.createComponent(Sorter);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
